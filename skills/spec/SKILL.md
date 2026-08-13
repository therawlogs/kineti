---
name: spec
description: Data contract and functional specification gate.
---

# Functional Specification Skill

This skill creates the formal functional specification and data contracts for the project. It runs after the initial discovery phase.

## 1. Data Quality Audit
Analyze the project's data requirements using the Q = C × A × T framework:
- **Completeness**: What data is required vs. optional?
- **Accuracy**: How is the data validated?
- **Timeliness**: Does the data need to be real-time, eventual, or batch updated?

## 2. Data-Routing Map
Map the physical flow of data through the system:
- User Input → Validation → Logic → Storage → API → UI.
Trace exactly how information travels from the client to the database and back.

## 3. Real Functional Specification Generation
Generate a comprehensive functional specification covering:
- **Field Schemas**: Table/model names, field types, required/optional flags, and strict constraints.
- **API Endpoint Signatures**: HTTP methods, route paths, structured request bodies, structured response bodies, and explicit error codes.
- **API Contract**: Define typed request/response schemas.
- **Error Handling**: Document specific fallback paths for service failures.
- **Edge Cases**: List abnormal usage patterns and how the system must handle them.

## 4. Save Public Specification
Write the generated specification to `<project_root>/spec.md`. This is a public repository file and must NOT be saved inside `.northstar/`.

## 5. HARD PAUSE GATE
Present the generated specification (`spec.md`) to the user.
**STOP.** Do not take any further action, do not save additional files, and do not proceed to planning until the user explicitly reviews and approves the specification schema.