use anyhow::Result;
use yazi_core::mgr::CdSource;
use yazi_macro::{act, succ};
use yazi_parser::VoidForm;
use yazi_shared::{data::Data, url::UrlLike};

use crate::{Actor, Ctx};

pub struct Enter;

impl Actor for Enter {
	type Form = VoidForm;

	const NAME: &str = "enter";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		if yazi_vfs::machines::is_root_url(cx.cwd()) {
			let Some(h) = cx.hovered() else { succ!() };
			let Some(slug) = yazi_vfs::machines::entry_slug_from_url(&h.url) else { succ!() };
			let url = yazi_vfs::machines::target_for_cached(&slug)?;
			return act!(mgr:cd, cx, (url, CdSource::Enter));
		}

		let Some(h) = cx.hovered().filter(|h| h.is_dir()) else { succ!() };

		let url = if h.url.is_search() { h.url.to_regular()? } else { h.url.clone() };

		act!(mgr:cd, cx, (url, CdSource::Enter))
	}
}
