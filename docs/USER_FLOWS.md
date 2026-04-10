# User Flows

Last updated: 2026-04-10

This document describes the main user journeys for Codex Continuity OS.

## Flow 1: First Run

Goal: user builds trust that the tool can see and organize their Codex history.

```mermaid
flowchart TD
    A[Clone repo and build ccx] --> B[Run ccx index]
    B --> C[Session archive scanned]
    C --> D[Local cache created]
    D --> E[Run ccx dashboard]
    E --> F[User sees project clusters]
    F --> G[User understands the product shape]
```

Success criteria:

- the index completes
- the dashboard opens
- the user sees recognizable repos/projects

## Flow 2: Resume A Repo After A Break

Goal: user recovers the right project context quickly.

```mermaid
flowchart TD
    A[Open repo folder] --> B[Run ccx dashboard --repo path]
    B --> C[Repo is preselected]
    C --> D[User sees recent sessions]
    D --> E[User reads goal and outcome]
    E --> F[User understands current state]
    F --> G[User takes next action or runs ccx pack]
```

Success criteria:

- correct repo is selected
- the best session is obvious
- the user can name the next action without reopening old chats manually

## Flow 3: Find The Chat About A Topic

Goal: user recovers a session by topic, not by memory of session id.

```mermaid
flowchart TD
    A[Select repo in dashboard] --> B[Press slash for search]
    B --> C[Type topic query]
    C --> D[Session list filters]
    D --> E[Select matching session]
    E --> F[Read detail pane]
```

Success criteria:

- user finds the correct session quickly
- filtered results feel clearly related to the query

## Flow 4: Compare Two Sessions

Goal: user understands whether two chats belong to the same continuity chain.

```mermaid
flowchart TD
    A[User has two candidate sessions] --> B[Run ccx compare a b]
    B --> C[Tool shows relation inference]
    C --> D[Tool shows common files and unique files]
    D --> E[User understands what changed]
```

Success criteria:

- relation is clear
- overlap and deltas are useful
- user can identify the later checkpoint

## Flow 5: Build Resume Pack For The Next Chat

Goal: user hands the next Codex session the right compact context.

```mermaid
flowchart TD
    A[User understands current repo state] --> B[Run ccx pack --repo path]
    B --> C[Tool selects latest checkpoint]
    C --> D[Tool selects richest context anchor]
    D --> E[Tool emits compact resume block]
    E --> F[User starts next Codex session with less friction]
```

Success criteria:

- pack feels compact
- files listed are actually relevant
- the suggested prompt is usable immediately

## Current UX Weakness In The Flows

The weakest part today is not the flow shape. It is the summary quality inside the flow.

That means:

- Flow 2 suffers when the selected session’s `goal` and `outcome` are shallow
- Flow 3 suffers when topic matching is correct but the displayed summary is weak
- Flow 5 suffers when the “best continuity summary” is only the last assistant reply

So the biggest leverage point is still:

- stronger whole-chat summarization

## Preferred Product Narrative

The product should increasingly feel like this:

1. open dashboard
2. pick project
3. inspect best session
4. search or compare if needed
5. pack and continue

That is the canonical user journey the rest of the product should reinforce.
