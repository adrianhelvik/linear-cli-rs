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
query {
    teams {
        nodes {
            id
            key
            name
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
query Users($first: Int) {
    users(first: $first) {
        nodes {
            id
            name
            email
            displayName
            active
        }
    }
}
"#;

pub const LABELS: &str = r#"
query Labels($first: Int) {
    issueLabels(first: $first) {
        nodes {
            id
            name
            color
        }
    }
}
"#;
