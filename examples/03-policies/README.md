# Policy Examples

This directory demonstrates policy and governance features in Ash.

These files illustrate policy patterns. Where an example uses a broader scenario-oriented style,
read it as reference material rather than as the canonical surface-syntax contract.

## Files

### 01-role-based.ash

Role-based access control (RBAC):

- Role definitions with authorities
- Role obligations and named-role policy decisions
- Capability requirements
- Policy decisions based on roles

### 02-time-based.ash

Time-based access control:

- Business hours restrictions
- Schedule-based permissions
- On-call and emergency overrides
- Audit logging with timestamps

## Key Concepts

1. **Roles** define authorities and obligations
2. **Policies** make permit/deny decisions
3. **Capabilities** specify required permissions
4. **Obligations** must be fulfilled
5. **Time-based rules** add temporal constraints

Canonical role reminders:

- Roles use the flat `authority` + `obligations` contract only.
- Approval remains an explicit named-role reference: `require_approval(role: X)`.

## Policy Decisions

- `permit` - Allow the action
- `deny` - Deny the action
- `require_approval(role: X)` - Require approval from a named role
- `escalate` - Escalate for separate handling
