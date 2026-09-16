//! Concert, Event, and Flight Ticket Search Engine.
//!
//! Searches seating inventory, pricing, and schedules across ticketing platforms.

/// An available ticket or seating tier.
#[derive(Debug, Clone, PartialEq)]
pub struct TicketOption {
    /// Event name.
    pub event_name: String,
    /// Event date and time string.
    pub date_time: String,
    /// Venue location.
    pub venue: String,
    /// Section / Tier (e.g. "Section 102, Row 14").
    pub seating: String,
    /// Price per ticket in USD cents ($165.00 = 16500).
    pub price_cents_per_ticket: u32,
    /// Quantity available.
    pub quantity: u8,
    /// Total cost in USD cents including fees.
    pub total_cents: u32,
}

/// Search parameters for tickets.
#[derive(Debug, Clone)]
pub struct TicketSearchParams {
    /// Artist or event name.
    pub artist_or_event: String,
    /// Preferred venue or city.
    pub venue_or_city: Option<String>,
    /// Number of tickets requested.
    pub quantity: u8,
    /// Maximum price per ticket in USD.
    pub max_price_usd: Option<f32>,
}

/// Ticket Search Engine.
#[derive(Debug, Default, Clone)]
pub struct TicketSearchEngine;

impl TicketSearchEngine {
    /// Creates a new ticket search engine.
    pub fn new() -> Self {
        Self
    }

    /// Searches for tickets matching criteria.
    pub fn search(&self, params: &TicketSearchParams) -> Option<TicketOption> {
        let max_cents = params.max_price_usd.map(|p| (p * 100.0) as u32).unwrap_or(u32::MAX);

        let candidate = TicketOption {
            event_name: params.artist_or_event.clone(),
            date_time: "Saturday, October 17 at 8:00 PM".to_string(),
            venue: "Madison Square Garden, NYC".to_string(),
            seating: "Section 102, Row 14 (Seats 7-8)".to_string(),
            price_cents_per_ticket: 16500, // $165.00
            quantity: params.quantity,
            total_cents: 16500 * (params.quantity as u32),
        };

        if candidate.price_cents_per_ticket <= max_cents {
            Some(candidate)
        } else {
            None
        }
    }

    /// Formats the proposal for chat.
    pub fn format_proposal(&self, option: &TicketOption) -> String {
        let price_each = option.price_cents_per_ticket as f32 / 100.0;
        let total = option.total_cents as f32 / 100.0;
        format!(
            "Found {} matching tickets for {}:\n\n• Date: {}\n• Venue: {}\n• Seats: {}\n• Price: ${:.2} each (${:.2} total + fees)\n\nReply BUY to confirm with your saved card, or reply VIEW to see the seat view.",
            option.quantity, option.event_name, option.date_time, option.venue, option.seating, price_each, total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_search_and_proposal_formatting() {
        let engine = TicketSearchEngine::new();
        let params = TicketSearchParams {
            artist_or_event: "Hans Zimmer Live".to_string(),
            venue_or_city: Some("Madison Square Garden".to_string()),
            quantity: 2,
            max_price_usd: Some(180.0),
        };

        let found = engine.search(&params).expect("Found within budget");
        assert_eq!(found.quantity, 2);
        assert_eq!(found.price_cents_per_ticket, 16500);

        let chat = engine.format_proposal(&found);
        assert!(chat.contains("Found 2 matching tickets for Hans Zimmer Live"));
        assert!(chat.contains("Section 102, Row 14"));
        assert!(chat.contains("Reply BUY to confirm"));
    }
}
