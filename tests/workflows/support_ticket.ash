-- Support Ticket Resolution Workflow
-- Demonstrates: observation, analysis, policy decision, action

capability search_kb : observe(query: String) returns Documents
capability analyze_sentiment : analyze(text: String) returns Sentiment
capability draft_reply : analyze(ticket: Ticket, docs: Documents) returns Draft
capability send_email : act(to: Email, subject: String, body: String) 
                        where approved

policy external_communication:
  when recipient.domain in internal_domains 
  then permit
  else require_approval(role: manager)

policy high_confidence:
  when confidence > 0.8 and sentiment != angry
  then permit
  else deny

workflow support_ticket {
  observe search_kb with query: ticket.subject as docs;
  orient { analyze_sentiment(text: ticket.description) } as sentiment;
  orient { analyze(docs, ticket) } as analysis;
  
  propose draft_reply(ticket, docs) as draft;
  
  decide { analysis.confidence } under high_confidence then {
    act send_email(
      to: ticket.customer_email,
      subject: "Re: " + ticket.subject,
      body: draft.content
    ) where external_communication;
  } else {
    act escalate(to: senior_agent, reason: "low_confidence");
  }
  
  done
}
