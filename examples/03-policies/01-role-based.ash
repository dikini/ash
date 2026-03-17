// Role-Based Access Control
//
// This workflow demonstrates role-based policies and permissions.

// Define roles with authorities and obligations
role admin {
    authority: [read, write, delete, approve],
    obligations: [audit_log],
    supervises: [manager, user]
}

role manager {
    authority: [read, write, approve],
    obligations: [review_reports],
    supervises: [user]
}

role user {
    authority: [read, write],
    obligations: [],
    supervises: []
}

role guest {
    authority: [read],
    obligations: [],
    supervises: []
}

// Define capabilities with required roles
capability read_document {
    effect: read,
    params: [doc_id: String],
    requires: any_role([admin, manager, user, guest])
}

capability write_document {
    effect: write,
    params: [doc_id: String, content: String],
    requires: any_role([admin, manager, user])
}

capability delete_document {
    effect: delete,
    params: [doc_id: String],
    requires: role(admin)
}

capability approve_change {
    effect: decide,
    params: [change_id: String],
    requires: any_role([admin, manager])
}

// Define policies
policy can_read {
    condition: user.role in [admin, manager, user, guest],
    decision: permit
}

policy can_write {
    condition: user.role in [admin, manager, user] && !document.locked,
    decision: permit
}

policy can_delete {
    condition: user.role == admin,
    decision: permit
}

workflow main {
    // Read document (allowed for all roles)
    observe read_document("doc-123") as document
    
    // Modify document (requires write permission)
    decide {
        if document.owner == current_user() {
            action "edit_document"
        } else {
            // Request approval
            propose request_approval(current_user(), "edit", "doc-123")
        }
    }
    
    // Check obligation
    check obliged admin audit_log
    
    // Log the access (obligation fulfillment)
    act audit_log {
        user: current_user(),
        action: "read",
        document: "doc-123",
        timestamp: now()
    }
    
    ret {
        document: document,
        access_granted: true,
        logged: true
    }
}
