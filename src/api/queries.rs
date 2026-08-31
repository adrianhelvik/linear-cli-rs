pub const VIEWER: &str = r#"
query {
    viewer {
        id
        name
        email
        displayName
    }
}
"#;

pub const TEAMS: &str = r#"
query Teams($first: Int, $after: String) {
    teams(first: $first, after: $after) {
        nodes {
            id
            key
            name
        }
        pageInfo {
            hasNextPage
            endCursor
        }
    }
}
"#;

pub const ISSUES: &str = r#"
query Issues($filter: IssueFilter, $first: Int, $after: String) {
    issues(filter: $filter, first: $first, after: $after, orderBy: updatedAt) {
        nodes {
            id
            identifier
            title
            priority
            url
            createdAt
            updatedAt
            state { id name type color }
            team { id key name }
            assignee { id name displayName }
            labels { nodes { id name color } }
            project { id name }
        }
        pageInfo {
            hasNextPage
            endCursor
        }
    }
}
"#;

pub const ISSUE: &str = r#"
query Issue($id: String!) {
    issue(id: $id) {
        id
        identifier
        title
        description
        priority
        estimate
        url
        branchName
        createdAt
        updatedAt
        state { id name type color }
        team { id key name }
        assignee { id name email displayName }
        labels { nodes { id name color } }
        project { id name }
        parent {
            id
            identifier
            title
            url
            priority
            state { id name type color }
            assignee { id name displayName }
        }
        children(first: 100) {
            nodes {
                id
                identifier
                title
                url
                priority
                state { id name type color }
                assignee { id name displayName }
            }
        }
        attachments(first: 50) {
            nodes {
                id
                title
                subtitle
                url
                sourceType
            }
        }
        comments(first: 100) {
            nodes {
                id
                body
                createdAt
                user { id name displayName }
                botActor { name }
                externalUser { id name displayName }
                parent { id }
            }
            pageInfo {
                hasNextPage
                endCursor
            }
        }
    }
}
"#;

pub const ISSUE_COMMENTS: &str = r#"
query IssueComments($id: String!, $after: String) {
    issue(id: $id) {
        comments(first: 100, after: $after) {
            nodes {
                id
                body
                createdAt
                user { id name displayName }
                botActor { name }
                externalUser { id name displayName }
                parent { id }
            }
            pageInfo {
                hasNextPage
                endCursor
            }
        }
    }
}
"#;

pub const ISSUE_SEARCH: &str = r#"
query SearchIssues($term: String!, $first: Int) {
    searchIssues(term: $term, first: $first) {
        nodes {
            id
            identifier
            title
            priority
            url
            state { id name type color }
            team { id key name }
            assignee { id name displayName }
            labels { nodes { id name color } }
        }
    }
}
"#;

pub const WORKFLOW_STATES: &str = r#"
query WorkflowStates($filter: WorkflowStateFilter) {
    workflowStates(filter: $filter) {
        nodes {
            id
            name
            type
            color
        }
    }
}
"#;

pub const USERS: &str = r#"
query Users($first: Int, $after: String) {
    users(first: $first, after: $after) {
        nodes {
            id
            name
            email
            displayName
            active
        }
        pageInfo {
            hasNextPage
            endCursor
        }
    }
}
"#;

pub const LABELS: &str = r#"
query Labels($first: Int, $after: String) {
    issueLabels(first: $first, after: $after) {
        nodes {
            id
            name
            color
        }
        pageInfo {
            hasNextPage
            endCursor
        }
    }
}
"#;
