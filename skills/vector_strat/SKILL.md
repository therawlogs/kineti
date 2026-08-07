---
name: /vector-strat
description: Designs search strategy, chunking rules, and retrieval setups for RAG systems and semantic memories, removing raw database dumps.
---

# Skill: /vector-strat (Step 09 - Retrieval Architecture Optimization)

## When to Use
When designing RAG engines, semantic memory layers, or knowledge bases.

## How to Use
Enter `/vector-strat` targeting data ingestion pipelines and chunking models.

## Sequencing
- **Phase**: `03_deterministic_ai_runtime`
- **Step**: 9 (Data infrastructure configuration pass).

## Protocol & Actions
- **Mode**: `RETRIEVAL_SURFACE_AREA_OPTIMIZATION` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/design`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** immediately design the database schema. Present the user with these retrieval architectural choices:

  1. **What is the preferred document chunking method?**
     * *Option A*: Hierarchical Parent-Child (Store small child chunks for vector matches, but return larger parent block context).
     * *Option B*: Sliding Window (Fixed chunk size like 512 tokens with 10% overlap).
     * *Option C*: Document Structure / Markdown Headings (Split only on semantic tags, sections, or functions).
     * *Option D*: Write-in.

  2. **Which search combination should be implemented?**
     * *Option A*: Hybrid Search (Keyword BM25 + Vector Cosine Similarity; high recall stability).
     * *Option B*: Pure Semantic Search (Vector matches only; depends heavily on custom model domain tuning).
     * *Option C*: Graph RAG (Entity extraction and relationship mapping).
     * *Option D*: Write-in.

  3. **Is algorithmic re-ranking required?**
     * *Option A*: Yes, Cross-Encoder (Provides maximum accuracy, adds ~100ms request latency).
     * *Option B*: No, use raw score thresholding (Fastest, depends on vector index metrics).
     * *Option C*: Write-in.

  Confirm selections before generating the retrieval diagram and specifications.

## Expected Output
- **Output type**: `LOW_NOISE_VECTOR_RETRI
<truncated 26 bytes>