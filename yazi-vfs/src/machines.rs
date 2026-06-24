use std::{collections::HashMap, io, path::PathBuf, process::Stdio, sync::OnceLock, time::Duration};

use parking_lot::Mutex;
use serde::Deserialize;
use tokio::{process::Command, time::timeout};
use yazi_fs::{cha::{Cha, ChaKind, ChaMode, ChaType}, file::File};
use yazi_shared::url::{UrlBuf, UrlLike};

use crate::config::{Service, ServiceSftp};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const COMMAND_RETRY_TIMEOUT: Duration = Duration::from_secs(20);
const ROUTE_TIMEOUT: Duration = Duration::from_secs(6);
const SFTP_CONNECT_TIMEOUT_SECS: u64 = 8;

#[cfg(windows)]
const ROOT_PATH: &str = r"C:\__open_yasa_machines__";
#[cfg(not(windows))]
const ROOT_PATH: &str = "/__open_yasa_machines__";

#[derive(Clone, Debug)]
pub struct Machine {
	pub machine_id:       String,
	pub friendly_name:    Option<String>,
	pub display_name:     String,
	pub platform:         Option<String>,
	pub workspace_path:   Option<String>,
	pub heartbeat_status: String,
	pub route:            String,
	pub local:            bool,
}

#[derive(Deserialize)]
struct Topology {
	#[serde(default)]
	local_machine_id: String,
	#[serde(default)]
	machines:         Vec<TopologyMachine>,
}

#[derive(Deserialize)]
struct TopologyMachine {
	machine_id:       String,
	#[serde(default)]
	friendly_name:    Option<String>,
	#[serde(default)]
	display_name:     Option<String>,
	#[serde(default)]
	platform:         Option<String>,
	#[serde(default)]
	workspace_path:   Option<String>,
	#[serde(default)]
	heartbeat_status: Option<String>,
	#[serde(default)]
	ssh:              Option<TopologySsh>,
}

#[derive(Deserialize)]
struct TopologySsh {
	#[serde(default)]
	route: Option<String>,
}

#[derive(Deserialize)]
struct Route {
	#[serde(default)]
	ok:             bool,
	#[serde(default)]
	route:          String,
	#[serde(default)]
	target:         Option<String>,
	#[serde(default)]
	command_target: Option<String>,
	#[serde(default)]
	local:          bool,
	#[serde(default)]
	warnings:       Option<Vec<String>>,
}

fn machine_cache() -> &'static Mutex<HashMap<String, Machine>> {
	static CACHE: OnceLock<Mutex<HashMap<String, Machine>>> = OnceLock::new();
	CACHE.get_or_init(Default::default)
}

fn service_cache() -> &'static Mutex<HashMap<String, (&'static str, &'static Service)>> {
	static CACHE: OnceLock<Mutex<HashMap<String, (&'static str, &'static Service)>>> =
		OnceLock::new();
	CACHE.get_or_init(Default::default)
}

fn machines_command() -> String {
	std::env::var("OPEN_YASA_MACHINES_COMMAND").unwrap_or_else(|_| "machines".to_owned())
}

async fn run_machines(args: &[&str], duration: Duration) -> io::Result<String> {
	let mut child = Command::new(machines_command());
	child.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

	let output = timeout(duration, child.output())
		.await
		.map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Open Machines command timed out"))??;
	if output.status.success() {
		return String::from_utf8(output.stdout)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
	}

	let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
	Err(io::Error::other(if stderr.is_empty() {
		format!("Open Machines command exited with {}", output.status)
	} else {
		stderr
	}))
}

async fn discover() -> io::Result<Vec<Machine>> {
	let raw = run_machines(&["topology", "--all", "--json"], COMMAND_TIMEOUT).await?;
	let topology = match parse_topology(&raw) {
		Ok(topology) => topology,
		Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
			tracing::debug!("Open Machines topology output was incomplete; retrying discovery");
			let raw = run_machines(&["topology", "--all", "--json"], COMMAND_RETRY_TIMEOUT).await?;
			parse_topology(&raw)?
		}
		Err(e) => return Err(e),
	};
	let local_cwd = std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned());

	Ok(
		topology
			.machines
			.into_iter()
			.map(|machine| {
				let route = machine.ssh.and_then(|ssh| ssh.route).unwrap_or_else(|| "unknown".to_owned());
				let local = machine.machine_id == topology.local_machine_id || route == "local";
				let display_name = machine.display_name.unwrap_or_else(|| machine.machine_id.clone());
				let workspace_path = if local {
					local_cwd.clone().or_else(|| machine.workspace_path.and_then(non_empty))
				} else {
					machine.workspace_path.and_then(non_empty)
				};
				Machine {
					machine_id: machine.machine_id,
					friendly_name: machine.friendly_name.and_then(non_empty),
					display_name,
					platform: machine.platform.and_then(non_empty),
					workspace_path,
					heartbeat_status: machine.heartbeat_status.unwrap_or_else(|| "unknown".to_owned()),
					route,
					local,
				}
			})
			.collect(),
	)
}

fn parse_topology(raw: &str) -> io::Result<Topology> {
	serde_json::from_str(raw).map_err(|e| {
		io::Error::new(
			if e.is_eof() { io::ErrorKind::UnexpectedEof } else { io::ErrorKind::InvalidData },
			format!("failed to parse Open Machines topology JSON ({} bytes): {e}", raw.len()),
		)
	})
}

fn non_empty(s: String) -> Option<String> {
	let s = s.trim().to_owned();
	(!s.is_empty()).then_some(s)
}

fn fallback_machine() -> Machine {
	let cwd = std::env::current_dir().ok().map(|p| p.to_string_lossy().into_owned());
	let display_name = std::env::var("HOSTNAME").ok().and_then(non_empty).unwrap_or_else(|| {
		std::env::var("COMPUTERNAME").ok().and_then(non_empty).unwrap_or_else(|| "local".to_owned())
	});

	Machine {
		machine_id: "local".to_owned(),
		friendly_name: None,
		display_name,
		platform: Some(std::env::consts::OS.to_owned()),
		workspace_path: cwd,
		heartbeat_status: "local".to_owned(),
		route: "local".to_owned(),
		local: true,
	}
}

pub fn root_url() -> UrlBuf { UrlBuf::from(PathBuf::from(ROOT_PATH)) }

pub fn is_root_url(url: &UrlBuf) -> bool { *url == root_url() }

pub fn root_cha() -> Cha { Cha::from_dummy(root_url(), Some(ChaType::Dir)) }

pub fn entry_slug_from_url(url: &UrlBuf) -> Option<String> {
	if !url.parent().is_some_and(|parent| parent == root_url()) {
		return None;
	}

	let name = url.name()?.to_string_lossy();
	name.split_ascii_whitespace().next().map(ToOwned::to_owned).filter(|s| !s.is_empty())
}

pub fn is_entry_url(url: &UrlBuf) -> bool { entry_slug_from_url(url).is_some() }

pub fn target_for_cached(slug: &str) -> io::Result<UrlBuf> {
	let machine = machine_cache()
		.lock()
		.get(slug)
		.cloned()
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Unknown machine: {slug}")))?;
	target_url(&machine)
}

pub async fn root_files() -> io::Result<Vec<File>> {
	let mut machines = discover().await.unwrap_or_else(|e| {
		tracing::debug!("Open Machines discovery failed, using local fallback: {e}");
		vec![fallback_machine()]
	});
	machines.sort_by(|a, b| a.machine_id.cmp(&b.machine_id));

	{
		let mut cache = machine_cache().lock();
		cache.clear();
		cache.extend(machines.iter().cloned().map(|machine| (machine.machine_id.clone(), machine)));
	}

	let root = root_url();
	machines
		.into_iter()
		.map(|machine| {
			let url = root.try_join(entry_name(&machine)).map_err(io::Error::other)?;
			Ok(machine_entry_file(url))
		})
		.collect()
}

fn machine_entry_file(url: UrlBuf) -> File {
	File {
		url,
		cha: Cha {
			kind: ChaKind::empty(),
			mode: ChaMode::T_DIR
				| ChaMode::U_READ
				| ChaMode::U_WRITE
				| ChaMode::U_EXEC
				| ChaMode::G_READ
				| ChaMode::G_EXEC
				| ChaMode::O_READ
				| ChaMode::O_EXEC,
			..Default::default()
		},
		link_to: None,
	}
}

fn entry_name(machine: &Machine) -> String {
	let mut parts = vec![machine.route.clone(), machine.heartbeat_status.clone()];
	if let Some(platform) = &machine.platform {
		parts.push(platform.clone());
	}

	let title = machine.friendly_name.as_ref().unwrap_or(&machine.display_name);
	let label = if title == &machine.machine_id {
		format!("{} [{}]", machine.machine_id, parts.join(" "))
	} else {
		format!("{} {} [{}]", machine.machine_id, clean_component(title), parts.join(" "))
	};
	clean_component(&label)
}

fn clean_component(s: &str) -> String {
	let cleaned: String = s
		.chars()
		.map(|c| match c {
			'/' | '\\' | '\0' | ':' | '"' | '<' | '>' | '|' | '?' | '*' => '-',
			c if c.is_control() => '-',
			c => c,
		})
		.collect();
	let cleaned = cleaned.trim();
	if cleaned.is_empty() { "unnamed".to_owned() } else { cleaned.to_owned() }
}

fn target_url(machine: &Machine) -> io::Result<UrlBuf> {
	let path = machine.workspace_path.as_deref().filter(|s| !s.trim().is_empty()).unwrap_or("/");
	if machine.local {
		return Ok(UrlBuf::from(PathBuf::from(path)));
	}

	if !path.starts_with('/') {
		return Err(io::Error::new(
			io::ErrorKind::InvalidData,
			format!("Remote machine {} has non-absolute workspace path: {path}", machine.machine_id),
		));
	}

	UrlBuf::try_from(format!("sftp://{}/{}", machine.machine_id, path))
		.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub async fn sftp_service(name: &str) -> io::Result<Option<(&'static str, &'static Service)>> {
	if let Some(service) = service_cache().lock().get(name).copied() {
		return Ok(Some(service));
	}

	let raw = match run_machines(
		&["route", "--machine", name, "--private-metadata", "--json"],
		ROUTE_TIMEOUT,
	)
	.await
	{
		Ok(raw) => raw,
		Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
		Err(e) => return Err(e),
	};
	let route: Route =
		serde_json::from_str(&raw).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

	if !route.ok {
		let message = route.warnings.unwrap_or_default().join("; ");
		return Err(io::Error::new(
			io::ErrorKind::NotFound,
			if message.is_empty() { format!("No Open Machines route for {name}") } else { message },
		));
	}
	if route.local || route.route == "local" {
		return Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"Local Open Machines entries use the local filesystem",
		));
	}

	let target = route
		.command_target
		.or(route.target)
		.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("No SSH target for {name}")))?;
	let (user, host, port) = parse_ssh_target(&target)?;

	let leaked_name: &'static str = Box::leak(name.to_owned().into_boxed_str());
	let service = Box::leak(Box::new(Service::Sftp(ServiceSftp::open_machines(
		host,
		user,
		port,
		SFTP_CONNECT_TIMEOUT_SECS,
	))));
	let resolved = (leaked_name, service as &'static Service);
	service_cache().lock().insert(name.to_owned(), resolved);
	Ok(Some(resolved))
}

fn parse_ssh_target(target: &str) -> io::Result<(String, String, u16)> {
	let (user, host_port) = target.split_once('@').ok_or_else(|| {
		io::Error::new(io::ErrorKind::InvalidData, "Open Machines SSH target does not include a user")
	})?;
	if user.is_empty() || host_port.is_empty() {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Open Machines SSH target"));
	}

	let (host, port) = match host_port.rsplit_once(':') {
		Some((host, port)) if port.bytes().all(|b| b.is_ascii_digit()) => {
			let port = port.parse::<u16>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
			(host.to_owned(), port)
		}
		_ => (host_port.to_owned(), 22),
	};

	Ok((user.to_owned(), host, port))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_ssh_target_defaults_port() {
		assert_eq!(
			parse_ssh_target("hasna@spark01").unwrap(),
			("hasna".to_owned(), "spark01".to_owned(), 22)
		);
	}

	#[test]
	fn parse_ssh_target_reads_port() {
		assert_eq!(
			parse_ssh_target("hasna@spark01:2222").unwrap(),
			("hasna".to_owned(), "spark01".to_owned(), 2222)
		);
	}

	#[test]
	fn entry_slug_uses_first_token() {
		let root = root_url();
		let url = root.try_join("spark02 [local online linux]").unwrap();
		assert_eq!(entry_slug_from_url(&url).as_deref(), Some("spark02"));
		assert!(is_entry_url(&url));
	}

	#[test]
	fn topology_parse_reports_truncated_json_as_eof() {
		match parse_topology(r#"{"local_machine_id":"spark02","machines":[{"#) {
			Ok(_) => panic!("truncated topology JSON parsed successfully"),
			Err(err) => assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof),
		}
	}

	#[test]
	fn machine_entries_render_as_real_directories() {
		let file = machine_entry_file(root_url().try_join("spark02 [local online linux]").unwrap());
		assert_eq!(*file.mode, ChaType::Dir);
		assert!(!file.is_dummy());
		assert!(!file.is_hidden());
	}

	#[test]
	fn clean_component_strips_unsafe_path_chars() {
		assert_eq!(clean_component("spark:03/alpha*"), "spark-03-alpha-");
	}

	#[test]
	fn remote_target_rejects_relative_workspace() {
		let err = target_url(&Machine {
			machine_id:       "spark01".to_owned(),
			friendly_name:    None,
			display_name:     "spark01".to_owned(),
			platform:         Some("linux".to_owned()),
			workspace_path:   Some("relative/workspace".to_owned()),
			heartbeat_status: "unknown".to_owned(),
			route:            "tailscale".to_owned(),
			local:            false,
		})
		.unwrap_err();

		assert_eq!(err.kind(), io::ErrorKind::InvalidData);
	}
}
