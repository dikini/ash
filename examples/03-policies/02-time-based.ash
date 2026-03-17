// Time-Based Policies
//
// This workflow demonstrates time-based access control and scheduling.

// Define business hours
const BUSINESS_HOURS_START = 9   // 9 AM
const BUSINESS_HOURS_END = 17    // 5 PM
const WEEKEND_DAYS = [6, 7]      // Saturday, Sunday (ISO 8601)

// Define roles with time restrictions
role operator {
    authority: [monitor, acknowledge],
    schedule: business_hours
}

role oncall {
    authority: [monitor, acknowledge, escalate, resolve],
    schedule: always
}

role auditor {
    authority: [read, generate_reports],
    schedule: weekdays
}

// Time-based policies
policy during_business_hours {
    condition: is_business_hours(),
    decision: permit
}

policy emergency_override {
    condition: is_emergency() || is_oncall(current_user()),
    decision: permit
}

policy maintenance_window {
    condition: is_scheduled_maintenance() && is_maintenance_role(current_user()),
    decision: permit
}

capability access_critical_system {
    effect: act,
    params: [system: String],
    requires: role(oncall) || (role(operator) && during_business_hours)
}

capability generate_audit_report {
    effect: read,
    params: [start_date: Time, end_date: Time],
    requires: role(auditor)
}

workflow main {
    // Get current time context
    orient {
        let currentTime = now()
        let isBusinessHours = is_business_hours()
        let isWeekend = day_of_week(currentTime) in WEEKEND_DAYS
        let user = current_user()
    } as context
    
    // Check if operation is allowed based on time
    decide {
        if context.isBusinessHours {
            // Normal operation
            action "standard_processing"
        } else if is_oncall(user) {
            // Oncall can access 24/7
            action "emergency_processing"
        } else {
            // Outside hours, no access
            ret {
                error: "Access denied: outside business hours",
                retry_after: next_business_hours_start()
            }
        }
    }
    
    // Log access with timestamp for audit
    act log_access {
        user: context.user,
        timestamp: context.currentTime,
        authorized: true,
        basis: if context.isBusinessHours { "business_hours" } else { "oncall" }
    }
    
    // Schedule follow-up during business hours if needed
    if !context.isBusinessHours {
        oblige operator {
            review_access_log {
                access_id: access_id(),
                review_by: next_business_day(9, 0)
            }
        }
    }
    
    ret {
        timestamp: context.currentTime,
        business_hours: context.isBusinessHours,
        access_granted: true
    }
}

// Helper functions (would be defined in standard library)
fn is_business_hours() -> Bool {
    let hour = hour_of_day(now())
    let day = day_of_week(now())
    hour >= BUSINESS_HOURS_START && hour < BUSINESS_HOURS_END && day not in WEEKEND_DAYS
}

fn next_business_hours_start() -> Time {
    // Returns the start of next business hours period
}

fn next_business_day(hour: Int, minute: Int) -> Time {
    // Returns the specified time on the next business day
}
