pub struct Templates;

impl Templates {
    pub fn get_requirements_template() -> &'static str {
        r#"# Requirements

## Technologies
- Technology stack to be defined

## Dependencies
- Dependencies to be listed

## Environment Setup
- Development environment requirements
- Build tools and configurations

## Performance Requirements
- Performance criteria and benchmarks
"#
    }
    
    pub fn get_design_template() -> &'static str {
        r#"# Design Specifications

## UI/UX Guidelines
- Design patterns to be defined

## Architecture
- System architecture to be documented

## User Flow
- User interaction patterns

## Accessibility
- Accessibility requirements and guidelines
"#
    }
    
    pub fn get_features_template() -> &'static str {
        r#"# Features

## Core Features
- Core functionality to be listed

## Future Features
- Planned enhancements

## User Stories
- User scenarios and requirements

## Acceptance Criteria
- Definition of done for each feature
"#
    }
    
    pub fn get_structure_template() -> &'static str {
        r#"# Project Structure

## Directory Layout
```
project/
├── src/
│   ├── components/
│   ├── services/
│   └── utils/
├── docs/
└── tests/
```

## File Organization
- File naming conventions
- Module structure
- Import/export patterns

## Build Configuration
- Build tools and scripts
- Environment configurations
"#
    }
    
    pub fn get_tasklist_template() -> &'static str {
        r#"# Task List

## To Do
- [ ] Task 1

## In Progress
- [ ] Task 2

## Done
- [x] Task 3
"#
    }
    
    pub fn get_claude_md_template() -> &'static str {
        r#"# Project Context

## Requirements
@include requirements.md

## Design
@include design.md

## Features
@include features.md

## Structure
@include structure.md

## Task List
@include tasklist.md
"#
    }
}