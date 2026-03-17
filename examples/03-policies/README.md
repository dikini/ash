# Policy Examples

This directory demonstrates policy and governance features in Ash.

## Files

### 01-role-based.ash
Role-based access control (RBAC):
- Role definitions with authorities
- Role hierarchies and supervision
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

## Policy Decisions

- `permit` - Allow the action
- `deny` - Deny the action
- `require_approval(role)` - Require approval from role
- `escalate` - Escalate to supervisor
