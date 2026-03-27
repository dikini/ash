// Code Review Workflow
//
// A workflow for processing pull request reviews with policy enforcement.
// This is a reference-oriented scenario example; canonical role and policy contracts live in
// `docs/spec/` and keep approval roles as flat named references.

// Capabilities
capability fetch_pr {
    effect: read,
    params: [repo: String, pr_number: Int],
    returns: PullRequest
}

capability fetch_diff {
    effect: read,
    params: [repo: String, pr_number: Int],
    returns: Diff
}

capability run_checks {
    effect: act,
    params: [repo: String, commit: String],
    returns: CheckResults
}

capability request_review {
    effect: act,
    params: [repo: String, pr_number: Int, reviewers: [String]],
    requires: role(author) || role(maintainer)
}

capability merge_pr {
    effect: act,
    params: [repo: String, pr_number: Int],
    requires: role(maintainer)
}

capability post_comment {
    effect: act,
    params: [repo: String, pr_number: Int, message: String],
    requires: any_role([author, reviewer, maintainer])
}

// Roles
role maintainer {
    capabilities: [merge, approve, request_changes, bypass_checks]
    obligations: []
}

role reviewer {
    capabilities: [approve, request_changes, comment]
    obligations: [respond_to_re_review]
}

role author {
    capabilities: [update_pr, respond_to_comments, request_review]
    obligations: [address_review_comments]
}

role ci_system {
    capabilities: [run_checks, post_status, auto_merge_patch]
    obligations: [all_checks_pass]
}

// Policies
policy require_review {
    condition: pr.lines_changed > 10 || pr.affected_files > 1 || pr.is_security_related,
    decision: require_approval(role: reviewer)
}

policy require_maintainer_for_large_changes {
    condition: pr.lines_changed > 500 || pr.affected_core,
    decision: require_approval(role: maintainer)
}

policy auto_merge_patch {
    condition: pr.is_patch && pr.all_checks_pass && pr.approvals >= 1,
    decision: permit
}

policy block_on_security_issues {
    condition: pr.security_scan.found_issues,
    decision: deny
}

policy require_tests {
    condition: pr.has_code_changes && !pr.has_test_changes && !pr.test_exemption,
    decision: require_approval(role: maintainer)
}

// Main workflow
workflow review_pull_request {
    // OBSERVE: Fetch PR data
    observe fetch_pr(repo, pr_number) as pr
    observe fetch_diff(repo, pr_number) as diff
    
    // ORIENT: Analyze the PR
    orient {
        // Analyze code changes
        let analysis = {
            lines_changed: diff.additions + diff.deletions,
            files_changed: length(diff.files),
            has_tests: any(diff.files, is_test_file),
            has_docs: any(diff.files, is_doc_file),
            complexity_score: calculate_complexity(diff),
            security_concerns: scan_for_security_issues(diff),
            breaking_changes: detect_breaking_changes(diff)
        }
        
        // Determine review requirements
        let needsTests = analysis.has_code_changes && !analysis.has_tests
        let needsDocs = analysis.has_api_changes && !analysis.has_docs
        let needsSecurityReview = analysis.security_concerns.found || pr.is_security_related
        let needsMaintainerReview = analysis.lines_changed > 500 || analysis.breaking_changes
        
        // Calculate risk score
        let riskScore = calculate_risk_score(pr, analysis)
        
        {
            pr: pr,
            analysis: analysis,
            needs_tests: needsTests,
            needs_docs: needsDocs,
            needs_security_review: needsSecurityReview,
            needs_maintainer_review: needsMaintainerReview,
            risk_score: riskScore
        }
    } as reviewContext
    
    // Run automated checks
    observe run_checks(repo, pr.head_commit) as checkResults
    
    // DECIDE: Apply policies
    decide {
        if checkResults.failed {
            action "block_checks_failed"
        } else if reviewContext.needs_security_review {
            action "require_security_review"
        } else if reviewContext.needs_maintainer_review {
            action "require_maintainer_approval"
        } else if reviewContext.risk_score < 30 && pr.approvals >= 1 {
            action "auto_merge_eligible"
        } else if reviewContext.risk_score < 50 {
            action "standard_review"
        } else {
            action "thorough_review_required"
        }
    }
    
    // ACT: Execute based on decision
    if action == "block_checks_failed" {
        act post_comment(repo, pr_number, format_check_failures(checkResults))
        act update_pr_status(repo, pr_number, "failure", "Checks failed")
        
        ret {
            status: "blocked",
            reason: "checks_failed",
            details: checkResults.failures
        }
    } else if action == "require_security_review" {
        act request_review(repo, pr_number, find_security_reviewers())
        act post_comment(repo, pr_number, "🔒 Security review required for this change.")
        act add_label(repo, pr_number, "security-review")
        
        ret {
            status: "pending",
            reason: "security_review_required",
            required_approvers: find_security_reviewers()
        }
    } else if action == "auto_merge_eligible" {
        // Auto-merge if enabled
        if repo.auto_merge_enabled {
            act merge_pr(repo, pr_number)
            act post_comment(repo, pr_number, "✅ Auto-merged: All checks passed and approved.")
            
            ret {
                status: "merged",
                method: "auto"
            }
        } else {
            act post_comment(repo, pr_number, "✅ Ready to merge: All requirements met.")
            
            ret {
                status: "ready",
                reason: "all_requirements_met"
            }
        }
    } else {
        // Standard review process
        let reviewers = if reviewContext.needs_maintainer_review {
            find_maintainers()
        } else {
            find_code_owners(diff.files)
        }
        
        act request_review(repo, pr_number, reviewers)
        
        // Post helpful context
        orient {
            let comment = generate_review_guidance(reviewContext)
        } as guidance
        
        act post_comment(repo, pr_number, guidance.comment)
        
        // Create obligations for reviewers
        for reviewer in reviewers {
            oblige reviewer respond_to_re_review {
                pr: pr_number,
                deadline: business_hours_from_now(24),
                priority: if reviewContext.risk_score > 70 { "high" } else { "normal" }
            }
        }
        
        ret {
            status: "review_requested",
            reviewers: reviewers,
            risk_score: reviewContext.risk_score,
            required_checks: [
                if reviewContext.needs_tests { "add_tests" } else { null },
                if reviewContext.needs_docs { "add_docs" } else { null }
            ]
        }
    }
}

// Helper functions
fn is_test_file(path: String) -> Bool {
    path.contains("_test.") || path.contains("/tests/") || path.contains("__tests__")
}

fn is_doc_file(path: String) -> Bool {
    path.ends_with(".md") || path.contains("/docs/")
}

fn calculate_complexity(diff: Diff) -> Int {
    // Calculate cyclomatic complexity of changes
}

fn scan_for_security_issues(diff: Diff) -> SecurityScan {
    // Run security scanning
}

fn detect_breaking_changes(diff: Diff) -> Bool {
    // Detect API breaking changes
}

fn calculate_risk_score(pr: PullRequest, analysis: Analysis) -> Int {
    let score = 0
    score = score + (analysis.lines_changed / 10)
    score = score + (if analysis.security_concerns.found { 50 } else { 0 })
    score = score + (if analysis.breaking_changes { 30 } else { 0 })
    score = score + (if !analysis.has_tests { 20 } else { 0 })
    score = score + (if pr.author.is_first_time { 10 } else { 0 })
    min(100, score)
}

fn format_check_failures(results: CheckResults) -> String {
    // Format check failures for comment
}

fn find_security_reviewers() -> [String] {
    // Get security team members
}

fn find_maintainers() -> [String] {
    // Get maintainers for the repo
}

fn find_code_owners(files: [File]) -> [String] {
    // Get code owners for affected files
}

fn generate_review_guidance(ctx: ReviewContext) -> String {
    // Generate helpful review guidance
}

fn business_hours_from_now(hours: Int) -> Time {
    // Calculate time after specified business hours
}
