//! Dialog layering system for proper rendering order
//! 
//! This module handles the layering and positioning of dialogs,
//! ensuring they render in the correct order and don't overlap incorrectly.

use super::types::{DialogId, DialogLayout};
use ratatui::layout::Rect;

/// Represents a dialog layer for rendering
#[derive(Debug, Clone)]
pub struct DialogLayer {
    /// Dialog identifier
    dialog_id: DialogId,
    
    /// Layout information for the dialog
    layout: DialogLayout,
    
    /// Whether this dialog currently has focus
    is_focused: bool,
    
    /// Z-index for rendering order (higher = on top)
    z_index: i32,
    
    /// Whether the dialog is currently visible
    is_visible: bool,
    
    /// Animation state (for future animation support)
    animation_progress: f32,
}

impl DialogLayer {
    /// Create a new dialog layer
    pub fn new(
        dialog_id: DialogId,
        layout: DialogLayout,
        is_focused: bool,
        z_index: i32,
    ) -> Self {
        Self {
            dialog_id,
            layout,
            is_focused,
            z_index,
            is_visible: true,
            animation_progress: 1.0,
        }
    }
    
    /// Get the dialog ID
    pub fn dialog_id(&self) -> &DialogId {
        &self.dialog_id
    }
    
    /// Get the layout information
    pub fn layout(&self) -> &DialogLayout {
        &self.layout
    }
    
    /// Check if the dialog has focus
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
    
    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }
    
    /// Get the z-index
    pub fn z_index(&self) -> i32 {
        self.z_index
    }
    
    /// Set the z-index
    pub fn set_z_index(&mut self, z_index: i32) {
        self.z_index = z_index;
    }
    
    /// Check if the dialog is visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    
    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }
    
    /// Get animation progress (0.0 = invisible, 1.0 = fully visible)
    pub fn animation_progress(&self) -> f32 {
        self.animation_progress
    }
    
    /// Set animation progress
    pub fn set_animation_progress(&mut self, progress: f32) {
        self.animation_progress = progress.clamp(0.0, 1.0);
    }
    
    /// Check if a point is within the dialog area
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        let area = &self.layout.dialog_area;
        x >= area.x && 
        x < area.x + area.width && 
        y >= area.y && 
        y < area.y + area.height
    }
    
    /// Check if this layer overlaps with another layer
    pub fn overlaps_with(&self, other: &DialogLayer) -> bool {
        let a = &self.layout.dialog_area;
        let b = &other.layout.dialog_area;
        
        !(a.x + a.width <= b.x || 
          b.x + b.width <= a.x || 
          a.y + a.height <= b.y || 
          b.y + b.height <= a.y)
    }
    
    /// Get the effective area considering animation
    pub fn effective_area(&self) -> Rect {
        if self.animation_progress >= 1.0 {
            return self.layout.dialog_area;
        }
        
        // Scale area based on animation progress
        let area = &self.layout.dialog_area;
        let scale = self.animation_progress;
        
        let scaled_width = (area.width as f32 * scale) as u16;
        let scaled_height = (area.height as f32 * scale) as u16;
        
        let x_offset = (area.width - scaled_width) / 2;
        let y_offset = (area.height - scaled_height) / 2;
        
        Rect {
            x: area.x + x_offset,
            y: area.y + y_offset,
            width: scaled_width,
            height: scaled_height,
        }
    }
    
    /// Update the layout
    pub fn update_layout(&mut self, layout: DialogLayout) {
        self.layout = layout;
    }
}

/// Dialog layer manager for organizing multiple layers
#[derive(Debug, Default)]
pub struct LayerManager {
    layers: Vec<DialogLayer>,
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }
    
    /// Add a new layer
    pub fn add_layer(&mut self, layer: DialogLayer) {
        self.layers.push(layer);
        self.sort_layers();
    }
    
    /// Remove a layer by dialog ID
    pub fn remove_layer(&mut self, dialog_id: &DialogId) -> Option<DialogLayer> {
        if let Some(index) = self.layers.iter().position(|layer| layer.dialog_id == *dialog_id) {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }
    
    /// Get a layer by dialog ID
    pub fn get_layer(&self, dialog_id: &DialogId) -> Option<&DialogLayer> {
        self.layers.iter().find(|layer| layer.dialog_id == *dialog_id)
    }
    
    /// Get a mutable layer by dialog ID
    pub fn get_layer_mut(&mut self, dialog_id: &DialogId) -> Option<&mut DialogLayer> {
        self.layers.iter_mut().find(|layer| layer.dialog_id == *dialog_id)
    }
    
    /// Get all layers in rendering order (sorted by z-index)
    pub fn layers(&self) -> &[DialogLayer] {
        &self.layers
    }
    
    /// Get all layers in rendering order (mutable)
    pub fn layers_mut(&mut self) -> &mut [DialogLayer] {
        &mut self.layers
    }
    
    /// Sort layers by z-index (lower z-index rendered first)
    fn sort_layers(&mut self) {
        self.layers.sort_by_key(|layer| layer.z_index);
    }
    
    /// Find the topmost layer at a given point
    pub fn layer_at_point(&self, x: u16, y: u16) -> Option<&DialogLayer> {
        // Iterate in reverse order (topmost first)
        self.layers
            .iter()
            .rev()
            .find(|layer| layer.is_visible && layer.contains_point(x, y))
    }
    
    /// Find all layers that overlap with a given area
    pub fn layers_in_area(&self, area: Rect) -> Vec<&DialogLayer> {
        self.layers
            .iter()
            .filter(|layer| {
                layer.is_visible && {
                    let layer_area = &layer.layout.dialog_area;
                    !(layer_area.x + layer_area.width <= area.x ||
                      area.x + area.width <= layer_area.x ||
                      layer_area.y + layer_area.height <= area.y ||
                      area.y + area.height <= layer_area.y)
                }
            })
            .collect()
    }
    
    /// Get the number of visible layers
    pub fn visible_count(&self) -> usize {
        self.layers.iter().filter(|layer| layer.is_visible).count()
    }
    
    /// Check if any layers are visible
    pub fn has_visible_layers(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_visible)
    }
    
    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }
    
    /// Update all layer layouts for a new terminal size
    pub fn update_layouts_for_size(&mut self, terminal_size: Rect) {
        for layer in &mut self.layers {
            // Recalculate layout for new terminal size
            // This would require access to dialog configs, so in practice
            // this should be handled by the DialogManager
        }
    }
    
    /// Set focus to a specific layer
    pub fn set_focus(&mut self, dialog_id: &DialogId, focused: bool) {
        if let Some(layer) = self.get_layer_mut(dialog_id) {
            layer.set_focused(focused);
        }
    }
    
    /// Get the focused layer
    pub fn focused_layer(&self) -> Option<&DialogLayer> {
        self.layers.iter().find(|layer| layer.is_focused)
    }
    
    /// Get the focused layer (mutable)
    pub fn focused_layer_mut(&mut self) -> Option<&mut DialogLayer> {
        self.layers.iter_mut().find(|layer| layer.is_focused)
    }
    
    /// Get the topmost layer (highest z-index)
    pub fn topmost_layer(&self) -> Option<&DialogLayer> {
        self.layers
            .iter()
            .filter(|layer| layer.is_visible)
            .max_by_key(|layer| layer.z_index)
    }
    
    /// Get the bottommost layer (lowest z-index)
    pub fn bottommost_layer(&self) -> Option<&DialogLayer> {
        self.layers
            .iter()
            .filter(|layer| layer.is_visible)
            .min_by_key(|layer| layer.z_index)
    }
    
    /// Bring a layer to front (increase z-index to be highest + 1)
    pub fn bring_to_front(&mut self, dialog_id: &DialogId) {
        if let Some(max_z) = self.layers.iter().map(|layer| layer.z_index).max() {
            if let Some(layer) = self.get_layer_mut(dialog_id) {
                layer.set_z_index(max_z + 1);
                self.sort_layers();
            }
        }
    }
    
    /// Send a layer to back (decrease z-index to be lowest - 1)
    pub fn send_to_back(&mut self, dialog_id: &DialogId) {
        if let Some(min_z) = self.layers.iter().map(|layer| layer.z_index).min() {
            if let Some(layer) = self.get_layer_mut(dialog_id) {
                layer.set_z_index(min_z - 1);
                self.sort_layers();
            }
        }
    }
}