use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

pub type CoreEventSender = UnboundedSender<CoreEvent>;

#[derive(Serialize, Debug)]
pub enum CoreEvent {}
