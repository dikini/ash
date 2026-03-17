// Customer Support Ticket Workflow
//
// A real-world workflow for processing customer support tickets
// using the OODA pattern with policy-based routing.

// Capabilities
capability fetch_ticket {
    effect: read,
    params: [ticket_id: String],
    returns: Ticket
}

capability fetch_customer {
    effect: read,
    params: [customer_id: String],
    returns: Customer
}

capability fetch_knowledge_base {
    effect: read,
    params: [query: String],
    returns: [Article]
}

capability send_notification {
    effect: act,
    params: [recipient: String, message: String],
    requires: role(agent) || role(supervisor)
}

capability escalate_ticket {
    effect: act,
    params: [ticket_id: String, reason: String],
    requires: role(agent)
}

capability resolve_ticket {
    effect: act,
    params: [ticket_id: String, resolution: String],
    requires: role(agent) || role(supervisor)
}

// Roles
role supervisor {
    authority: [view_all, assign, escalate, resolve, refund],
    supervises: [agent]
}

role agent {
    authority: [view_assigned, respond, resolve, escalate],
    obligations: [respond_within_sla]
}

role system {
    authority: [auto_classify, auto_respond, route],
    obligations: [log_all_actions]
}

// Policies
policy urgent_priority {
    condition: ticket.priority == "urgent" || ticket.sentiment < -0.5,
    decision: escalate
}

policy vip_customer {
    condition: customer.tier == "enterprise" || customer.tier == "premium",
    decision: require_approval(supervisor)
}

policy auto_resolve {
    condition: ticket.confidence > 0.9 && ticket.category == "common_issue",
    decision: permit
}

policy sla_breach_imminent {
    condition: ticket.time_to_sla < 300,  // 5 minutes
    decision: escalate
}

// Main workflow
workflow process_ticket {
    // OBSERVE: Gather all relevant data
    observe fetch_ticket(ticket_id) as ticket
    observe fetch_customer(ticket.customer_id) as customer
    observe fetch_knowledge_base(ticket.subject) as articles
    
    // ORIENT: Analyze and classify
    orient {
        // Calculate priority score
        let priorityScore = calculate_priority(ticket, customer)
        
        // Find relevant articles
        let relevantArticles = filter_relevant(articles, ticket)
        
        // Analyze sentiment
        let sentiment = analyze_sentiment(ticket.description)
        
        // Determine category
        let category = classify_ticket(ticket, relevantArticles)
        
        // Check SLA status
        let slaStatus = check_sla(ticket)
        
        {
            ticket: ticket,
            customer: customer,
            priority_score: priorityScore,
            sentiment: sentiment,
            category: category,
            sla_status: slaStatus,
            suggested_articles: relevantArticles
        }
    } as analysis
    
    // DECIDE: Apply policies
    decide {
        if analysis.sla_status.breached {
            action "immediate_escalate"
        } else if analysis.priority_score >= 90 || analysis.sentiment < -0.7 {
            action "escalate_to_supervisor"
        } else if analysis.category == "refund_request" {
            action "request_supervisor_approval"
        } else if analysis.category.can_auto_resolve && analysis.priority_score < 50 {
            action "auto_respond"
        } else {
            action "assign_to_agent"
        }
    }
    
    // ACT: Execute based on decision
    if action == "auto_respond" {
        act send_notification(customer.email, generate_auto_response(analysis))
        act resolve_ticket(ticket_id, "auto_resolved")
    } else if action == "assign_to_agent" {
        orient {
            let bestAgent = find_best_agent(analysis.category, analysis.priority_score)
        } as assignment
        
        act assign_ticket(ticket_id, assignment.bestAgent.id)
        act send_notification(assignment.bestAgent.email, "New ticket assigned: " + ticket_id)
        
        // Create obligation for agent response
        oblige agent respond_within_sla {
            ticket: ticket_id,
            deadline: sla_deadline(ticket),
            max_response_time: ticket.sla.response_time
        }
    } else if action == "escalate_to_supervisor" || action == "immediate_escalate" {
        act escalate_ticket(ticket_id, "High priority or SLA breach")
        
        // Notify supervisor
        par {
            act send_notification(find_supervisor().email, "URGENT: Escalated ticket " + ticket_id)
            act send_notification(customer.email, "Your ticket has been escalated to our specialist team.")
        }
    }
    
    // Log all actions for audit
    act log_ticket_processing {
        ticket_id: ticket_id,
        analysis: analysis,
        decision: action,
        timestamp: now(),
        processor: current_user()
    }
    
    ret {
        ticket_id: ticket_id,
        status: if action == "auto_respond" { "resolved" } else { "in_progress" },
        assigned_to: if action == "assign_to_agent" { assignment.bestAgent.id } else { null },
        action_taken: action
    }
}

// Helper function definitions
fn calculate_priority(ticket: Ticket, customer: Customer) -> Int {
    let basePriority = ticket.priority_value
    let tierBonus = if customer.tier == "enterprise" { 30 }
                   else if customer.tier == "premium" { 20 }
                   else { 0 }
    let slaBonus = if ticket.time_to_sla < 600 { 20 } else { 0 }
    min(100, basePriority + tierBonus + slaBonus)
}

fn filter_relevant(articles: [Article], ticket: Ticket) -> [Article] {
    // Filter articles by relevance score
}

fn analyze_sentiment(text: String) -> Float {
    // Sentiment analysis
}

fn classify_ticket(ticket: Ticket, articles: [Article]) -> Category {
    // ML-based classification
}

fn check_sla(ticket: Ticket) -> SlaStatus {
    // Check SLA compliance
}

fn find_best_agent(category: Category, priority: Int) -> Agent {
    // Find available agent with matching skills
}

fn generate_auto_response(analysis: Analysis) -> String {
    // Generate personalized response from articles
}
