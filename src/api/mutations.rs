pub const ISSUE_CREATE: &str = r#"
mutation IssueCreate($input: IssueCreateInput!) {
    issueCreate(input: $input) {
        success
        issue {
            id
            identifier
            title
            url
        }
    }
}
"#;

pub const ISSUE_UPDATE: &str = r#"
mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
        success
        issue {
            id
            identifier
            title
            url
        }
    }
}
"#;

pub const COMMENT_CREATE: &str = r#"
mutation CommentCreate($input: CommentCreateInput!) {
    commentCreate(input: $input) {
        success
        comment {
            id
            body
            createdAt
            user { id name displayName }
        }
    }
}
"#;
