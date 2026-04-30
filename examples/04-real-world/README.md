# Real-World Examples

This directory preserves practical, real-world workflow scenario sketches.

Phase 107 classifies the `.ash` files in this directory as **reference-only**. They are useful scenario/design material, but they use broad historical OODA, role, policy, obligation, and action forms that are not current executable conformance syntax. They are excluded from the executable example corpus until rewritten against the current parser/typechecker surface.

## Files

### customer-support.ash
A complete customer support ticket workflow featuring:
- Multi-source data observation (ticket, customer, knowledge base)
- AI-powered analysis (sentiment, classification, priority)
- Policy-based routing (urgent, VIP, SLA)
- Role-based assignment (agent, supervisor)
- Obligation tracking for SLA compliance

### code-review.ash
A pull request review workflow featuring:
- Automated code analysis (diff parsing, complexity)
- Security scanning integration
- Policy enforcement (required reviews, tests, docs)
- Auto-merge for low-risk changes
- Obligation creation for reviewers

## Key Patterns

### OODA Loop
Both workflows follow the OODA pattern:
- **Observe**: Gather data from multiple sources
- **Orient**: Analyze and enrich data
- **Decide**: Apply policies to determine action
- **Act**: Execute with appropriate capabilities

### Policy-Driven Routing
Decisions are made by applying policies:
- Priority-based routing
- Risk-based review requirements
- Automatic vs. manual processing

### Role-Based Access
Different roles have different authorities:
- **Customer Support**: supervisor > agent > system
- **Code Review**: maintainer > reviewer > author

### Audit and Compliance
Both workflows include comprehensive logging:
- All actions logged with timestamps
- Obligations tracked and enforced
- Audit trails for compliance
