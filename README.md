# Intent-Segregation-Cybersecurity-Architecture-for-AI
Researching a system for preventing prompt injection by separating user intent from user content

## 1. Overview
This project implements an intent-first, schema-driven security architecture designed to mitigate prompt injection and unsafe LLM actions. The system separates:
- User Intent (what the user wants the system to do)
- User Content (text, documents, contextual details)

The architecture uses:

- Independent intent parsers
- A voting-based intent validator
- An intent comparator
- A trusted intent generator
- A processing engine that accepts only typed, structured intents
- An intent ledger for auditing
- Optional human approval for elevated-risk actions

This design is especially suitable for narrow, well-defined AI applications, such as B2B consulting automation, customer support tools, or workflow agents.

## 2. System Architecture
```
User Input
   │
   ├──► Malicious Input Detector (Might Implement)
   │
   ├──► Parser Ensemble (P1, P2, P3...)
   │         │
   │         └──► Voting Module
   │
   ├──► Intent Comparator ◄── Provider Config
   │
   ├─── if mismatch → Elevated Privilege Request → Human Approval
   │
   ├──► Trusted Intent Generator
   │
   ├──► Processing Engine
   │         │
   │         └──► Intent Ledger
   │
   └──► Response
```

<img width="75%" alt="Screenshot 2025-11-22 163153" src="https://github.com/user-attachments/assets/256bcc99-0609-48a8-b166-75735769b1af" />


## 3. Modules
### 3.1 Malicious Input Detector

Purpose: Quickly reject obviously harmful or malformed inputs before deeper processing.

Implementation:
- Light syntactic checks (regex-based).
- Block commands like rm -rf /, cloud resource manipulation, SQL injection patterns, etc.
- ML classifier (optional) to flag “attack-like” inputs.

Output: either “clean” or “blocked”.

### 3.2 Intent Parser Ensemble

Several independent modules parse free-form user text into a structured intent JSON.

Parsers:

1. Deterministic Parser

  - Rule-based mapping for actions.
  - Keyword → enum resolution.
  - Zero hallucination; highest trust.
2. LLM Parser #1
  - Extracts action, topic, expertise, constraints.
  - Temperature set to 0.
  - Constrained to JSON schema.
3. LLM Parser #2
  - Same schema, but different model or system prompt.
  - Increases independence.
Output: structured JSON for each parser.

### 3.3 Voting Module

Compares outputs from all parsers.

Logic:

- If all parsers agree → high confidence.
- If small discrepancies → fallback to deterministic parser + request user confirmation.
- If major discrepancies → escalate to human approval.

Output: canonical parsed intent.

### 3.4 Intent Comparator

Compares:

- Parsed user intent vs
- Provider-defined allowed intents (capability config)

```
Provider Config Example:

{
  "allowed_actions": ["find_experts", "summarize", "draft_proposal"],
  "allowed_expertise": ["ml", "embedded", "security"],
  "max_budget": 50000
}
```

Comparator checks:

- action is allowed (enum)
- expertise subset is allowed
- parameters are within constraints
- topic is semantically similar to allowed domain

Decision:

- match → approve
- soft mismatch → require confirmation
- hard mismatch → block or escalate

### 3.5 Trusted Intent Generator

Produces a canonical, sanitized, and signed JSON object.

Guarantees:
- Only allowed fields appear.
- No raw user content is injected.
- content_refs are references to sanitized documents.

Example output:
```
{
  "action": "find_experts",
  "topic_id": "supply_chain_risk",
  "expertise": ["security"],
  "constraints": {
    "max_budget": 20000
  },
  "content_refs": ["doc_1321"]
}
```

### 3.6 Processing Engine

Executes trusted intents via function calls—not raw prompts.

Implementation Requirements:

- All operations are typed.
- Processing agents must consume only trusted intents.
- No “free-form LLM” call can execute privileged actions.

Example callable:
```
get_experts({ topic_id, expertise, max_budget })
```

### 3.7 Intent Ledger

Append-only log storing:
- User input
- Parsed intents from ensemble
- Comparator decisions
- Trusted intent
- Processing outputs
- Any privilege elevation events

Writable only; immutable once saved.

Great for:

- Auditing
- Forensics
- Explaining decisions
- Research evaluation

### 3.8 Human Supervision Module

Triggered when:
- Intent mismatch
- Model disagreement
- High-risk actions
- Unusual parameter patterns

Human gets a UI showing:
- Raw user input
- Parsed JSON intents
- Diffs
- Explanation of mismatch

They can approve / deny / correct.

## 4. Example Folder Structure

```
.
├── core/
│   ├── parsers/
│   ├── comparator/
│   ├── intent_generator/
│   ├── malicious_detector/
│   ├── processing_engine/
│   ├── ledger/
│   └── supervision/
│
├── config/
│   ├── provider_config.json
│   └── schema.json
│
├── api/
│   └── server.py
│
├── tests/
│   └── redteam/
│
└── README.md
```
