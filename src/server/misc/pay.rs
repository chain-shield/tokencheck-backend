use stripe::{
    CreateCheckoutSessionLineItemsPriceDataRecurring,
    CreateCheckoutSessionLineItemsPriceDataRecurringInterval,
};

use crate::server::dtos::pay::RecurringInfo;

impl From<&RecurringInfo> for Option<CreateCheckoutSessionLineItemsPriceDataRecurring> {
    fn from(info: &RecurringInfo) -> Self {
        let int = match info.interval.as_str() {
            "day" => CreateCheckoutSessionLineItemsPriceDataRecurringInterval::Day,
            "week" => CreateCheckoutSessionLineItemsPriceDataRecurringInterval::Week,
            "month" => CreateCheckoutSessionLineItemsPriceDataRecurringInterval::Month,
            "year" => CreateCheckoutSessionLineItemsPriceDataRecurringInterval::Year,
            _ => return None,
        };
        Some(CreateCheckoutSessionLineItemsPriceDataRecurring {
            interval: int,
            interval_count: Some(info.interval_count),
        })
    }
}
