-- Multi-Agent Research Workflow
-- Demonstrates: parallel deliberation, synthesis, collaboration

role analyst {
  authority: [search_literature, extract_findings],
  obligations: [cite_sources],
  supervises: []
}

role critic {
  authority: [review_findings, identify_gaps],
  obligations: [verify_claims],
  supervises: []
}

role synthesizer {
  authority: [synthesize_themes, draft_report],
  obligations: [ensure_coherence],
  supervises: [analyst, critic]
}

capability search_literature : observe(query: String) returns Papers
capability extract_findings : analyze(papers: Papers) returns Findings
capability identify_gaps : analyze(papers: Papers) returns Gaps
capability identify_themes : analyze(papers: Papers) returns Themes
capability synthesize : analyze(inputs: List<Perspective>) returns Synthesis
capability draft_report : analyze(synthesis: Synthesis) returns Report
capability publish_report : act(report: Report) where peer_reviewed

workflow collaborative_research {
  observe search_literature with query: research_question as papers;
  
  par {
    with role: analyst do {
      orient extract_findings(papers) as findings
    };
    
    with role: critic do {
      orient identify_gaps(papers) as gaps
    };
    
    with role: synthesizer do {
      orient identify_themes(papers) as themes
    }
  } as perspectives;
  
  orient synthesize(perspectives) as synthesis;
  propose draft_report(synthesis) as report;
  
  oblige role: reviewer to verify_claims(report);
  
  decide { report.confidence > threshold } then {
    act publish_report(report) where peer_reviewed;
  } else {
    maybe {
      act request_feedback(report)
    } else {
      act archive_as_draft(report)
    }
  }
  
  done
}
