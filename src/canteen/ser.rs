use serde::Serialize;

use super::{CanteenId, Meta};

#[derive(Debug, Serialize)]
pub struct CanteenCompleteWithoutMeals<'c> {
    pub id: CanteenId,
    #[serde(flatten)]
    pub meta: &'c Meta,
}
