# Phase 3: Complete TUI

**Duration:** 3-4 weeks  
**Priority:** MEDIUM  
**Target Completion:** 90% of total porting

## Overview

Phase 3 transforms Goofy from a basic terminal application into a sophisticated, feature-rich TUI that rivals modern IDEs. This phase focuses on user experience, visual polish, and advanced interaction patterns.

## Critical Components

### 1. Advanced TUI Architecture

**Location:** `src/tui/`

**Component-Based Design:**
```rust
pub trait Component {
    fn render(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
    fn handle_event(&mut self, event: &Event) -> Result<EventResponse>;
    fn focus(&mut self) -> bool;
    fn unfocus(&mut self);
    fn is_focused(&self) -> bool;
}

pub enum EventResponse {
    Handled,
    NotHandled,
    Exit,
    SwitchPage(PageType),
}
```

### 2. Advanced Chat Interface

**Location:** `src/tui/components/chat/`

**Modular Chat Components:**
- `editor/` - Multi-line input with syntax highlighting
- `messages/` - Rich message display with formatting
- `sidebar/` - Session navigation and management
- `header/` - Chat metadata and controls
- `toolbar/` - Quick actions and tool selection

```rust
pub struct ChatEditor {
    pub content: String,
    pub cursor_position: usize,
    pub selection: Option<(usize, usize)>,
    pub syntax_highlighter: Option<SyntaxHighlighter>,
    pub completion_popup: Option<CompletionPopup>,
}

pub struct MessageRenderer {
    pub markdown_renderer: MarkdownRenderer,
    pub code_highlighter: CodeHighlighter,
    pub image_viewer: ImageViewer,
    pub scroll_state: ScrollState,
}
```

### 3. Modal System

**Location:** `src/tui/components/dialogs/`

**Dialog Types:**
- Confirmation dialogs
- Input dialogs
- File selection dialogs
- Settings dialogs
- Help dialogs

```rust
pub struct DialogManager {
    pub active_dialog: Option<Box<dyn Dialog>>,
    pub dialog_stack: Vec<Box<dyn Dialog>>,
}

pub trait Dialog: Component {
    fn title(&self) -> &str;
    fn can_cancel(&self) -> bool;
    fn on_confirm(&mut self) -> Result<DialogResult>;
    fn on_cancel(&mut self) -> Result<DialogResult>;
}
```

### 4. Advanced Navigation

**Location:** `src/tui/components/sidebar/`

**Sidebar Features:**
- Session tree with hierarchy
- Search functionality  
- Favorites and pinning
- Context menus
- Drag and drop

```rust
pub struct Sidebar {
    pub session_tree: SessionTree,
    pub search_box: SearchBox,
    pub selected_session: Option<String>,
    pub expanded_sessions: HashSet<String>,
    pub context_menu: Option<ContextMenu>,
}
```

### 5. Image and Media Support

**Location:** `src/tui/components/media/`

**Media Capabilities:**
- Image display (ASCII art conversion)
- File attachments
- Code snippets with syntax highlighting
- Diff visualization

```rust
pub struct ImageViewer {
    pub image_data: Vec<u8>,
    pub display_mode: ImageDisplayMode,
    pub zoom_level: f32,
    pub ascii_converter: AsciiConverter,
}

pub enum ImageDisplayMode {
    Ascii,
    Sixel,
    Kitty,
    Iterm2,
}
```

## Implementation Steps

### Step 1: Enhanced Chat Interface (Week 1)

1. **Multi-line editor with syntax highlighting**
   ```rust
   // src/tui/components/chat/editor.rs
   pub struct ChatEditor {
       buffer: String,
       cursor: CursorState,
       highlighter: SyntaxHighlighter,
       completion: CompletionEngine,
   }
   ```

2. **Rich message rendering**
   - Markdown support with syntect
   - Code block highlighting
   - Table rendering
   - Link highlighting

3. **Message interaction**
   - Copy to clipboard
   - Edit previous messages
   - Message threading
   - Reaction system

### Step 2: Sidebar and Navigation (Week 1-2)

1. **Session tree implementation**
   ```rust
   // src/tui/components/sidebar/tree.rs
   pub struct SessionTree {
       nodes: Vec<TreeNode>,
       selected: Option<usize>,
       expanded: HashSet<usize>,
   }
   ```

2. **Advanced navigation features**
   - Keyboard shortcuts
   - Search and filtering
   - Bulk operations
   - Session management

### Step 3: Modal Dialog System (Week 2)

1. **Dialog framework**
   ```rust
   // src/tui/components/dialogs/mod.rs
   pub struct DialogStack {
       dialogs: Vec<Box<dyn Dialog>>,
       backdrop: bool,
   }
   ```

2. **Common dialogs**
   - File picker
   - Confirmation prompts
   - Settings editor
   - About dialog

### Step 4: Advanced Rendering (Week 2-3)

1. **Image support**
   - ASCII art conversion
   - Terminal graphics protocols
   - Image caching
   - Format detection

2. **Enhanced text rendering**
   - Diff visualization
   - Table layouts
   - Progress indicators
   - Animations

### Step 5: Theming and Customization (Week 3-4)

1. **Theme system**
   ```rust
   // src/tui/themes/mod.rs
   pub struct Theme {
       pub colors: ColorScheme,
       pub styles: StyleMap,
       pub icons: IconSet,
   }
   ```

2. **Responsive layouts**
   - Window resizing
   - Adaptive components
   - Mobile-friendly layouts
   - Accessibility features

## Advanced Features

### 1. Keyboard Shortcuts System

```rust
// src/tui/keybindings.rs
pub struct KeyBindings {
    pub global: HashMap<KeyEvent, Action>,
    pub modal: HashMap<String, HashMap<KeyEvent, Action>>,
    pub context: HashMap<ComponentType, HashMap<KeyEvent, Action>>,
}

pub enum Action {
    Quit,
    NewSession,
    SwitchSession(usize),
    SendMessage,
    CopyMessage(usize),
    ToggleSidebar,
    OpenSettings,
    // ... more actions
}
```

### 2. Animation System

```rust
// src/tui/animations.rs
pub struct AnimationManager {
    pub active_animations: Vec<Box<dyn Animation>>,
    pub frame_rate: u64,
}

pub trait Animation {
    fn update(&mut self, delta_time: std::time::Duration) -> bool;
    fn render(&self, frame: &mut Frame, area: Rect);
}
```

### 3. Plugin System

```rust
// src/tui/plugins.rs
pub trait TuiPlugin {
    fn name(&self) -> &str;
    fn render_hook(&self, context: &RenderContext) -> Option<Widget>;
    fn event_hook(&self, event: &Event) -> Option<EventResponse>;
}
```

## Configuration Extensions

### Enhanced TUI Configuration

```json
{
  "tui": {
    "theme": "dark",
    "animations": true,
    "sidebar": {
      "width": 25,
      "auto_hide": false,
      "show_icons": true
    },
    "editor": {
      "syntax_highlighting": true,
      "line_numbers": true,
      "auto_complete": true,
      "vim_mode": false
    },
    "chat": {
      "message_limit": 1000,
      "auto_scroll": true,
      "timestamp_format": "%H:%M:%S",
      "markdown_rendering": true
    },
    "keybindings": {
      "quit": "Ctrl+Q",
      "new_session": "Ctrl+N",
      "switch_session": "Ctrl+Tab",
      "toggle_sidebar": "Ctrl+B"
    }
  }
}
```

## Testing Strategy

### Visual Testing
- Screenshot testing for layouts
- Theme consistency validation
- Responsive behavior testing
- Accessibility compliance

### Interaction Testing
- Keyboard navigation flows
- Mouse interaction testing
- Dialog state management
- Animation performance

### Performance Testing
- Large message history handling
- Memory usage optimization
- Render performance benchmarks
- Input latency measurements

## Success Criteria

- [ ] Complete chat interface with all features
- [ ] Functional sidebar with session management
- [ ] Modal dialog system working
- [ ] Image display capabilities
- [ ] Theme system implementation
- [ ] Advanced keyboard shortcuts
- [ ] Smooth animations
- [ ] Responsive design
- [ ] Performance optimization
- [ ] Accessibility features

## Dependencies

**New Crates:**
```toml
[dependencies]
# Enhanced TUI
tui-textarea = "0.4"
tui-tree-widget = "0.17"

# Image processing
image = "0.24"
sixel = "0.1"

# Animation
easing = "0.2"

# Clipboard
copypasta = "0.8"

# Terminal capabilities
term_size = "0.3"
terminal_size = "0.3"

# Accessibility
a11y = "0.1"
```

## Risk Mitigation

**Performance Risks:**
- Large message histories can slow rendering
- Implement virtual scrolling
- Add message pagination

**Memory Usage:**
- Images and animations consume memory
- Implement smart caching
- Add memory limits

**Terminal Compatibility:**
- Different terminals have varying capabilities
- Implement graceful degradation
- Test across terminal types

## Integration Points

**With Phase 1 & 2:**
- Display tool outputs in rich format
- Show LSP diagnostics in sidebar
- Integrate MCP tools in UI
- Permission prompts as dialogs

**With Configuration:**
- Theme persistence
- Keybinding customization
- Layout preferences
- Performance settings

This phase creates a world-class terminal user interface that showcases all of Goofy's capabilities in an intuitive and beautiful way.