//! # Kineti Actions (`kineti-actions`)
//!
//! Real-world actions and autonomous tool workflows:
//! - **[`shopping`]**: Price comparison across retailers (Amazon, Best Buy, B&H).
//! - **[`tickets`]**: Event, concert, and flight ticket discovery.
//! - **[`confirmation`]**: Two-step financial safety gate and [`FinancialCheckoutConnector`] enforcing cryptographic token gating.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod confirmation;
pub mod shopping;
pub mod tickets;

pub use confirmation::{
    ActionConfirmationGate, ConfirmationDecision, FinancialCheckoutConnector, PendingFinancialAction,
};
pub use shopping::{MerchantOffer, PriceComparisonEngine, PriceComparisonReport};
pub use tickets::{TicketOption, TicketSearchEngine, TicketSearchParams};

/// Prelude module for convenient access to kineti-actions types.
pub mod prelude {
    pub use crate::confirmation::{
        ActionConfirmationGate, ConfirmationDecision, FinancialCheckoutConnector, PendingFinancialAction,
    };
    pub use crate::shopping::{MerchantOffer, PriceComparisonEngine, PriceComparisonReport};
    pub use crate::tickets::{TicketOption, TicketSearchEngine, TicketSearchParams};
}
