-- Code Review Workflow with Role Separation
-- Demonstrates: roles, obligations, parallel execution
-- Reference-oriented example; canonical role contracts use only authority and obligations,
-- and approval roles remain flat named references in `docs/spec/`.

role drafter {
  authority: [read_code, create_pr, respond_to_comments],
  obligations: [ensure_tests_pass]
}

role reviewer {
  authority: [read_code, comment, request_changes, approve],
  obligations: [check_tests, check_security, review_logic]
}

capability fetch_pr : observe(pr_id: ID) returns PR
capability analyze_diff : analyze(pr: PR) returns Analysis
capability check_coverage : analyze(tests: TestSuite) returns Coverage
capability request_changes : act(pr: PR, comments: List<Comment>)
capability merge_pr : act(pr: PR) where all_checks_pass

workflow code_review {
  let pr = observe fetch_pr with pr_id: $input.pr_id;
  
  par {
    orient analyze_diff(pr) as diff_analysis;
    orient check_coverage(pr.tests) as coverage
  };
  
  oblige reviewer to check_tests(pr);
  oblige reviewer to check_security(pr);
  
  decide { coverage.percentage > 80 and diff_analysis.no_critical_issues } then {
    if diff_analysis.has_minor_issues then {
      act request_changes(pr, comments: diff_analysis.issues);
    } else {
      act merge_pr(pr) where reviewer_approved;
    }
  } else {
    act request_changes(
      pr, 
      comments: ["Coverage insufficient", "Critical issues found"]
    );
  }
  
  done
}
