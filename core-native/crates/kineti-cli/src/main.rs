//! # Kineti Native CLI Entrypoint
//!
//! Provides operational commands to test, run, and inspect the Kineti assistant.

use kineti_core::conversation::UserTier;
use kineti_core::spend::UserSpendQuota;
use kineti_gateway::{GatewayRouter, OutboundReply};
use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "chat" => run_interactive_chat(),
        "status" => run_status(),
        "test-all" => run_test_suite(),
        _ => print_help(),
    }
}

fn print_help() {
    println!(
        "\n=======================================================\n\
           Kineti — Proprietary Messaging AI Assistant\n\
           Website: getkineti.com | Pure Rust Nervous System\n\
         =======================================================\n\n\
         Usage:\n\
           kineti-cli chat       Start interactive chat testing session\n\
           kineti-cli status     Show memory and spend quota status\n\
           kineti-cli test-all   Run end-to-end verification checks\n"
    );
}

fn run_status() {
    let router = GatewayRouter::new();
    let quota = UserSpendQuota::new("local_dev_user", UserTier::Pro.daily_quota_microcents());

    println!("\n--- Kineti Assistant Status ---");
    println!("Architecture: Pure Native Rust (Zero Node/Python dependencies)");
    println!("Spend Quota: Limit ${:.2} | Spent ${:.2}",
        quota.limit_microcents() as f32 / 1_000_000.0,
        quota.spent_microcents() as f32 / 1_000_000.0,
    );
    println!("Active Memory Nodes: {}", router.memory.query_facts("local_dev_user", None).len());
    println!("Supported Channels: Apple iMessage (macOS bridge) & Meta WhatsApp Cloud API");
    println!("Real-World Actions: Active (Shopping, Tickets, Brave Search, FLUX.1)");
    println!("-------------------------------\n");
}

fn run_interactive_chat() {
    let router = GatewayRouter::new();
    let user_id = "local_dogfood_user";
    let quota = UserSpendQuota::new(user_id, UserTier::Pro.daily_quota_microcents());

    println!("\n=======================================================");
    println!("  Kineti Native Chat Session (Simulated iMessage/WhatsApp)");
    println!("  Type 'exit' to quit. Text naturally or try slangs!");
    println!("=======================================================\n");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("You > ");
    stdout.flush().unwrap();

    for line in stdin.lock().lines() {
        let text = match line {
            Ok(t) => t,
            Err(_) => break,
        };

        let trimmed = text.trim();
        if trimmed == "exit" || trimmed == "quit" {
            println!("Goodbye!");
            break;
        }

        if trimmed.is_empty() {
            print!("You > ");
            stdout.flush().unwrap();
            continue;
        }

        let reply = router.process_message(user_id, trimmed, None, &quota);
        match reply {
            OutboundReply::Reaction { emoji } => {
                println!("Kineti > [Reaction: {} attached to message bubble]", emoji);
            }
            OutboundReply::Text { body } => {
                println!("Kineti >\n{}", body);
            }
            OutboundReply::Image { media_url, caption } => {
                println!("Kineti > [Photo Delivered: {}]\nCaption: {}", media_url, caption);
            }
        }

        println!();
        print!("You > ");
        stdout.flush().unwrap();
    }
}

fn run_test_suite() {
    println!("\nRunning Kineti Verification Suite...");
    let router = GatewayRouter::new();
    let user_id = "test_verification_user";
    let quota = UserSpendQuota::new(user_id, UserTier::Pro.daily_quota_microcents());

    // 1. Test status update reflex
    let r1 = router.process_message(user_id, "leaving now, see you later", None, &quota);
    assert_eq!(r1, OutboundReply::Reaction { emoji: "⚡" });
    println!("  [1/5] Low-info status reaction (⚡): PASS");

    // 2. Test style-adaptive greeting
    let r2 = router.process_message(user_id, "yo", None, &quota);
    if let OutboundReply::Text { body } = r2 {
        assert!(body.contains("hey!"));
        println!("  [2/5] Style-adaptive greeting: PASS");
    } else {
        panic!("Failed greeting test");
    }

    // 3. Test memory capture
    let r3 = router.process_message(user_id, "remember that my manager is Sarah Chen", None, &quota);
    if let OutboundReply::Text { body } = r3 {
        assert!(body.to_lowercase().contains("sarah chen"));
        println!("  [3/5] Memory capture & style-matching: PASS");
    } else {
        panic!("Failed memory capture test");
    }

    // 4. Test price comparison
    let r4 = router.process_message(user_id, "Find me the best price on Sony headphones", None, &quota);
    if let OutboundReply::Text { body } = r4 {
        assert!(body.contains("Amazon: $328.00"));
        println!("  [4/5] Real-world price comparison: PASS");
    } else {
        panic!("Failed price comparison test");
    }

    // 5. Test ticket search & confirmation gate
    let r5 = router.process_message(user_id, "Look for 2 good tickets for Hans Zimmer", None, &quota);
    if let OutboundReply::Text { body } = r5 {
        assert!(body.contains("Reply BUY to confirm"));
        println!("  [5/5] Two-step ticket booking confirmation gate: PASS");
    } else {
        panic!("Failed ticket confirmation test");
    }

    println!("\nAll 5 verification flows passed successfully!\n");
}
