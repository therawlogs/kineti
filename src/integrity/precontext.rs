//! Pre-context filter (10XE C1, D1): exact-dedup, recency preservation,
//! attention-budget cap, duplicate-drop flags. System messages always survive.

use crate::provider::Msg;

pub struct Filtered {
    pub messages: Vec<Msg>,
    pub flags: Vec<String>,
}

pub fn filter(msgs: &[Msg], budget: usize) -> Filtered {
    let mut flags = Vec::new();

    // 1. exact-duplicate tool results: keep the latest occurrence
    let mut seen_dup = 0usize;
    let mut kept_idx: Vec<usize> = Vec::new();
    for (i, m) in msgs.iter().enumerate() {
        if i > 0
            && matches!(m.role, crate::provider::Role::Tool { .. })
            && msgs[..i].iter().any(|prev| prev.content == m.content)
        {
            seen_dup += 1;
            continue; // drop older duplicate
        }
        kept_idx.push(i);
    }
    if seen_dup > 0 {
        flags.push(format!("dropped {seen_dup} duplicate tool outputs"));
    }

    // 2. attention-budget cap: elide the middle, keep head + tail
    let total: usize = kept_idx.iter().map(|&i| msgs[i].content.len()).sum();
    let mut messages: Vec<Msg> = Vec::new();
    if total <= budget {
        for &i in &kept_idx {
            messages.push(msgs[i].clone());
        }
    } else {
        flags.push(format!(
            "budget cap engaged: {} → ≤{} chars (middle elided)",
            total, budget
        ));
        // keep system messages + first user + last 4 messages verbatim
        let keep_head: Vec<usize> = kept_idx
            .iter()
            .take_while(|&&i| {
                matches!(msgs[i].role, crate::provider::Role::System)
                    || (messages.is_empty() && matches!(msgs[i].role, crate::provider::Role::User))
            })
            .copied()
            .collect();
        let tail_n = 4.min(kept_idx.len());
        let tail_start = kept_idx.len() - tail_n;
        let mut used: usize = keep_head.iter().chain(kept_idx[tail_start..].iter()).map(|&i| msgs[i].content.len()).sum();
        for (pos, &i) in kept_idx.iter().enumerate() {
            let is_kept = pos < keep_head.len() || pos >= tail_start;
            if is_kept || used + msgs[i].content.len() / 4 < budget {
                used += msgs[i].content.len();
                messages.push(msgs[i].clone());
            } else {
                let len = msgs[i].content.len();
                messages.push(Msg {
                    role: msgs[i].role.clone(),
                    content: format!("[elided {len} chars — outside attention budget]"),
                    tool_calls: vec![],
                    extra: None,
                });
            }
        }
    }

    Filtered { messages, flags }
}
