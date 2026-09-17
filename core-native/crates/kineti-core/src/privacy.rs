//! # Privacy Controls & Model Training Opt-Out (`privacy`)
//!
//! Manages user privacy controls, including the "Improve Kineti for everyone"
//! model training opt-out toggle and outbound telemetry egress gating.
//!
//! Invariant: When model training opt-out is enabled, user prompts, context memory,
//! and session telemetry are strictly barred from outbound model fine-tuning collectors.

use std::sync::atomic::{AtomicBool, Ordering};

/// User privacy settings and model training controls.
#[derive(Debug)]
pub struct PrivacyController {
    /// Whether user opted out of model training ("Improve Kineti for everyone" = false).
    /// When true, data will NEVER be used for model training or outbound telemetry.
    model_training_opted_out: AtomicBool,
}

impl Default for PrivacyController {
    fn default() -> Self {
        Self::new(true) // Default to privacy-preserving: opted out unless explicitly enabled
    }
}

impl PrivacyController {
    /// Creates a new privacy controller with initial opt-out status.
    pub const fn new(opted_out: bool) -> Self {
        Self {
            model_training_opted_out: AtomicBool::new(opted_out),
        }
    }

    /// Returns whether the user has opted out of model training.
    pub fn is_training_opted_out(&self) -> bool {
        self.model_training_opted_out.load(Ordering::SeqCst)
    }

    /// Sets the model training opt-out state.
    /// `true` means user opts OUT (no training).
    /// `false` means user allows telemetry to improve models.
    pub fn set_training_opt_out(&self, opted_out: bool) {
        self.model_training_opted_out.store(opted_out, Ordering::SeqCst);
    }

    /// Invariant check: validates whether an outbound telemetry payload is allowed.
    /// Returns `Ok(())` if allowed, or `Err("Telemetry blocked by user model training opt-out")`.
    pub fn verify_telemetry_egress(&self) -> Result<(), &'static str> {
        if self.is_training_opted_out() {
            Err("Outbound telemetry blocked by user model training opt-out preference")
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_controller_defaults_to_opted_out() {
        let privacy = PrivacyController::default();
        assert!(privacy.is_training_opted_out());
        assert!(privacy.verify_telemetry_egress().is_err());
    }

    #[test]
    fn test_privacy_controller_toggle() {
        let privacy = PrivacyController::new(true);
        assert!(privacy.is_training_opted_out());

        privacy.set_training_opt_out(false);
        assert!(!privacy.is_training_opted_out());
        assert!(privacy.verify_telemetry_egress().is_ok());

        privacy.set_training_opt_out(true);
        assert!(privacy.is_training_opted_out());
        assert!(privacy.verify_telemetry_egress().is_err());
    }
}
