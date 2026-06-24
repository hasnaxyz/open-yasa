use anyhow::Result;
use yazi_actor::Ctx;
use yazi_boot::{ARGS, BOOT};
use yazi_config::YAZI;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, strand::StrandLike, url::UrlLike};
use yazi_vfs::machines;

use crate::Actor;

pub struct Bootstrap;

impl Actor for Bootstrap {
	type Form = VoidForm;

	const NAME: &str = "bootstrap";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if ARGS.entries.is_empty()
			&& ARGS.cwd_file.is_none()
			&& ARGS.chooser_file.is_none()
			&& YAZI.open_yasa.machines_layer.get()
		{
			act!(mgr:cd, cx, (machines::root_url(), CdSource::Tab))?;
			succ!();
		}

		cx.mgr.tabs.resize_with(BOOT.files.len(), Default::default);

		for (i, file) in BOOT.files.iter().enumerate().rev() {
			cx.tab = i;
			if file.is_empty() {
				act!(mgr:cd, cx, (BOOT.cwds[i].clone(), CdSource::Tab))?;
			} else if let Ok(u) = BOOT.cwds[i].try_join(file) {
				act!(mgr:reveal, cx, (u, CdSource::Tab))?;
			}
		}

		succ!();
	}
}
