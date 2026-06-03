use crate::domain::{error::AppError, model::ActionResult, model::TailscaleStatus};

pub trait StatusReader {
    fn read_status(&self) -> Result<TailscaleStatus, AppError>;
}

pub trait TailscaleController {
    fn up(&self) -> Result<ActionResult, AppError>;
    fn down(&self) -> Result<ActionResult, AppError>;
}
