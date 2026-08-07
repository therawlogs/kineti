---
name: /scrape
description: Ephemeral data ingestion utility. Runs a programmatic browser crawl on documentation targets, stripping tracking and noise.
---

# Skill: /scrape (Ephemeral Data Ingestion Utility)

## When to Use
When accurate external technical schemas or documentation data is required.

## How to Use
Enter `/scrape` followed by the target documentation URI.

## Sequencing
- **Phase**: `07_data_tools`
- **Step**: Ephemeral data ingestion utility.

## Protocol & Actions
- **Mode**: `ASYMMETRIC_SCRAPING_ENGINE` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** crawl immediately. Confirm parameters with the user:

  1. **Select the crawling depth option:**
     * *Option A*: Single Page Scan (Only scrape the exact target URI specified).
     * *Option B*: Recursive Site Scrape (Scrape the target page and any links matching the same domain up to depth 2).
     * *Option C*: Selector-Targeted Scan (Scrape only the content inside a specific CSS id/class like `#api-reference`).
     * *Option D*: Write-in.

  2. **Select the target output format:**
     * *Option A*: Dense Markdown (Stripped of tags; optimal for developer context window injection).
     * *Option B*: Structured JSON (Create mapping matrices containing schemas, endpoints, and method names).
     * *Option C*: Write-in.

  Verify configurations before initiating browser crawlers.

## Expected Output
- **Output type**: `STRUCTURED_LOW_DECAY_JSON_OR_MARKDOWN_CONTEXT`