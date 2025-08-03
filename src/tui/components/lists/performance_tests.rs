//! Performance tests for list components with large datasets
//! 
//! This module provides comprehensive performance testing for list
//! components to ensure smooth operation with thousands of items.

use anyhow::Result;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use super::{
    VirtualList, ListConfig, ListItem, FilterConfig, ScrollState,
    virtualization::{VirtualScrollConfig, ViewportInfo},
};
use crate::tui::themes::Theme;

/// Performance test suite for list components
pub struct ListPerformanceTest {
    /// Test configurations
    configs: Vec<TestConfig>,
    
    /// Test results
    results: Vec<TestResult>,
    
    /// Performance thresholds
    thresholds: PerformanceThresholds,
}

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Test name
    pub name: String,
    
    /// Number of items to test
    pub item_count: usize,
    
    /// Item complexity (simple, medium, complex)
    pub item_complexity: ItemComplexity,
    
    /// Virtual scrolling enabled
    pub virtual_scrolling: bool,
    
    /// Filtering enabled
    pub filtering: bool,
    
    /// Number of test iterations
    pub iterations: usize,
    
    /// Viewport size
    pub viewport_size: (u16, u16),
}

/// Item complexity levels
#[derive(Debug, Clone, Copy)]
pub enum ItemComplexity {
    /// Simple text items
    Simple,
    /// Items with multiple fields
    Medium,
    /// Complex items with nested data
    Complex,
}

/// Test result
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test configuration
    pub config: TestConfig,
    
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    
    /// Whether test passed
    pub passed: bool,
    
    /// Error message if failed
    pub error: Option<String>,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Initial render time
    pub initial_render_time: Duration,
    
    /// Average scroll time
    pub average_scroll_time: Duration,
    
    /// Filter time (if applicable)
    pub filter_time: Option<Duration>,
    
    /// Memory usage estimate
    pub memory_usage_mb: f64,
    
    /// Frame rate (FPS)
    pub frame_rate: f64,
    
    /// Scroll smoothness score (0-100)
    pub smoothness_score: f64,
    
    /// Additional timing metrics
    pub timing_details: HashMap<String, Duration>,
}

/// Performance thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum initial render time
    pub max_initial_render_ms: u64,
    
    /// Maximum scroll time
    pub max_scroll_time_ms: u64,
    
    /// Maximum filter time
    pub max_filter_time_ms: u64,
    
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    
    /// Minimum frame rate
    pub min_frame_rate: f64,
    
    /// Minimum smoothness score
    pub min_smoothness_score: f64,
}

/// Test data generator
pub struct TestDataGenerator;

/// Mock list item for testing
#[derive(Debug, Clone)]
pub struct TestListItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub complexity: ItemComplexity,
}

impl ListPerformanceTest {
    /// Create a new performance test suite
    pub fn new() -> Self {
        Self {
            configs: Self::default_test_configs(),
            results: Vec::new(),
            thresholds: PerformanceThresholds::default(),
        }
    }
    
    /// Run all performance tests
    pub async fn run_all_tests(&mut self) -> Result<TestSummary> {
        let start_time = Instant::now();
        let mut passed = 0;
        let mut failed = 0;
        
        for config in self.configs.clone() {
            println!("Running test: {}", config.name);
            
            match self.run_single_test(config.clone()).await {
                Ok(result) => {
                    if result.passed {
                        passed += 1;
                        println!("✅ {} passed", config.name);
                    } else {
                        failed += 1;
                        println!("❌ {} failed: {}", config.name, 
                               result.error.as_deref().unwrap_or("Unknown error"));
                    }
                    self.results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    println!("💥 {} errored: {}", config.name, e);
                    self.results.push(TestResult {
                        config,
                        metrics: PerformanceMetrics::default(),
                        passed: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        let total_time = start_time.elapsed();
        
        Ok(TestSummary {
            total_tests: self.configs.len(),
            passed,
            failed,
            total_time,
            results: self.results.clone(),
        })
    }
    
    /// Run a single performance test
    async fn run_single_test(&self, config: TestConfig) -> Result<TestResult> {
        let mut metrics = PerformanceMetrics::default();
        let mut timing_details = HashMap::new();
        
        // Generate test data
        let data_gen_start = Instant::now();
        let test_items = TestDataGenerator::generate_items(
            config.item_count,
            config.item_complexity,
        );
        timing_details.insert("data_generation".to_string(), data_gen_start.elapsed());
        
        // Create list component
        let list_config = ListConfig {
            virtual_scrolling: config.virtual_scrolling,
            viewport_height: config.viewport_size.1,
            item_height: 3, // Assume 3 rows per item
            buffer_size: 10,
            enable_filtering: config.filtering,
            enable_sorting: true,
            enable_selection: true,
            show_scrollbar: true,
        };
        
        let mut virtual_list = VirtualList::new(list_config);
        
        // Test initial render
        let render_start = Instant::now();
        virtual_list.set_items(test_items.clone());
        metrics.initial_render_time = render_start.elapsed();
        
        // Test scrolling performance
        metrics.average_scroll_time = self.test_scrolling_performance(
            &mut virtual_list,
            config.iterations,
        ).await?;
        
        // Test filtering if enabled
        if config.filtering {
            let filter_start = Instant::now();
            virtual_list.set_filter("test".to_string());
            metrics.filter_time = Some(filter_start.elapsed());
        }
        
        // Calculate memory usage estimate
        metrics.memory_usage_mb = self.estimate_memory_usage(&test_items);
        
        // Calculate frame rate and smoothness
        let (frame_rate, smoothness) = self.measure_rendering_performance(
            &virtual_list,
            Duration::from_millis(1000),
        ).await?;
        
        metrics.frame_rate = frame_rate;
        metrics.smoothness_score = smoothness;
        metrics.timing_details = timing_details;
        
        // Check if test passed
        let passed = self.check_performance_thresholds(&metrics);
        let error = if passed {
            None
        } else {
            Some(self.generate_failure_message(&metrics))
        };
        
        Ok(TestResult {
            config,
            metrics,
            passed,
            error,
        })
    }
    
    /// Test scrolling performance
    async fn test_scrolling_performance(
        &self,
        virtual_list: &mut VirtualList<TestListItem>,
        iterations: usize,
    ) -> Result<Duration> {
        let mut total_time = Duration::ZERO;
        let item_count = virtual_list.total_items();
        
        for i in 0..iterations {
            let scroll_position = (i * item_count / iterations).min(item_count.saturating_sub(1));
            
            let start = Instant::now();
            virtual_list.scroll_to(scroll_position);
            total_time += start.elapsed();
            
            // Simulate a small delay between scrolls
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        
        Ok(total_time / iterations as u32)
    }
    
    /// Measure rendering performance
    async fn measure_rendering_performance(
        &self,
        virtual_list: &VirtualList<TestListItem>,
        test_duration: Duration,
    ) -> Result<(f64, f64)> {
        let start_time = Instant::now();
        let mut frame_count = 0;
        let mut frame_times = Vec::new();
        
        while start_time.elapsed() < test_duration {
            let frame_start = Instant::now();
            
            // Simulate rendering
            let _visible_items = virtual_list.visible_items();
            
            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);
            frame_count += 1;
            
            // Target 60 FPS
            tokio::time::sleep(Duration::from_micros(16667)).await;
        }
        
        let total_time = start_time.elapsed();
        let frame_rate = frame_count as f64 / total_time.as_secs_f64();
        
        // Calculate smoothness score based on frame time consistency
        let smoothness_score = self.calculate_smoothness_score(&frame_times);
        
        Ok((frame_rate, smoothness_score))
    }
    
    /// Calculate smoothness score based on frame time variance
    fn calculate_smoothness_score(&self, frame_times: &[Duration]) -> f64 {
        if frame_times.is_empty() {
            return 0.0;
        }
        
        let target_frame_time = Duration::from_micros(16667); // 60 FPS
        let mut variance_sum = 0.0;
        
        for &frame_time in frame_times {
            let diff = frame_time.as_secs_f64() - target_frame_time.as_secs_f64();
            variance_sum += diff * diff;
        }
        
        let variance = variance_sum / frame_times.len() as f64;
        let std_dev = variance.sqrt();
        
        // Convert to 0-100 score (lower std_dev = higher score)
        let score = ((1.0 / (1.0 + std_dev * 1000.0)) * 100.0).min(100.0);
        score
    }
    
    /// Estimate memory usage
    fn estimate_memory_usage(&self, items: &[TestListItem]) -> f64 {
        let base_size = std::mem::size_of::<TestListItem>();
        let mut total_size = items.len() * base_size;
        
        for item in items {
            total_size += item.title.len();
            total_size += item.description.len();
            total_size += item.tags.iter().map(|tag| tag.len()).sum::<usize>();
            total_size += item.metadata.iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>();
        }
        
        total_size as f64 / 1024.0 / 1024.0 // Convert to MB
    }
    
    /// Check if metrics meet performance thresholds
    fn check_performance_thresholds(&self, metrics: &PerformanceMetrics) -> bool {
        metrics.initial_render_time.as_millis() <= self.thresholds.max_initial_render_ms as u128
            && metrics.average_scroll_time.as_millis() <= self.thresholds.max_scroll_time_ms as u128
            && metrics.filter_time.map_or(true, |t| t.as_millis() <= self.thresholds.max_filter_time_ms as u128)
            && metrics.memory_usage_mb <= self.thresholds.max_memory_mb
            && metrics.frame_rate >= self.thresholds.min_frame_rate
            && metrics.smoothness_score >= self.thresholds.min_smoothness_score
    }
    
    /// Generate failure message
    fn generate_failure_message(&self, metrics: &PerformanceMetrics) -> String {
        let mut issues = Vec::new();
        
        if metrics.initial_render_time.as_millis() > self.thresholds.max_initial_render_ms as u128 {
            issues.push(format!(
                "Initial render too slow: {}ms > {}ms",
                metrics.initial_render_time.as_millis(),
                self.thresholds.max_initial_render_ms
            ));
        }
        
        if metrics.average_scroll_time.as_millis() > self.thresholds.max_scroll_time_ms as u128 {
            issues.push(format!(
                "Scrolling too slow: {}ms > {}ms",
                metrics.average_scroll_time.as_millis(),
                self.thresholds.max_scroll_time_ms
            ));
        }
        
        if let Some(filter_time) = metrics.filter_time {
            if filter_time.as_millis() > self.thresholds.max_filter_time_ms as u128 {
                issues.push(format!(
                    "Filtering too slow: {}ms > {}ms",
                    filter_time.as_millis(),
                    self.thresholds.max_filter_time_ms
                ));
            }
        }
        
        if metrics.memory_usage_mb > self.thresholds.max_memory_mb {
            issues.push(format!(
                "Memory usage too high: {:.2}MB > {:.2}MB",
                metrics.memory_usage_mb,
                self.thresholds.max_memory_mb
            ));
        }
        
        if metrics.frame_rate < self.thresholds.min_frame_rate {
            issues.push(format!(
                "Frame rate too low: {:.1}fps < {:.1}fps",
                metrics.frame_rate,
                self.thresholds.min_frame_rate
            ));
        }
        
        if metrics.smoothness_score < self.thresholds.min_smoothness_score {
            issues.push(format!(
                "Smoothness too low: {:.1} < {:.1}",
                metrics.smoothness_score,
                self.thresholds.min_smoothness_score
            ));
        }
        
        issues.join("; ")
    }
    
    /// Default test configurations
    fn default_test_configs() -> Vec<TestConfig> {
        vec![
            TestConfig {
                name: "Small Dataset - Simple Items".to_string(),
                item_count: 100,
                item_complexity: ItemComplexity::Simple,
                virtual_scrolling: false,
                filtering: false,
                iterations: 10,
                viewport_size: (80, 24),
            },
            TestConfig {
                name: "Medium Dataset - Simple Items".to_string(),
                item_count: 1_000,
                item_complexity: ItemComplexity::Simple,
                virtual_scrolling: true,
                filtering: false,
                iterations: 20,
                viewport_size: (80, 24),
            },
            TestConfig {
                name: "Large Dataset - Simple Items".to_string(),
                item_count: 10_000,
                item_complexity: ItemComplexity::Simple,
                virtual_scrolling: true,
                filtering: false,
                iterations: 50,
                viewport_size: (80, 24),
            },
            TestConfig {
                name: "Very Large Dataset - Simple Items".to_string(),
                item_count: 100_000,
                item_complexity: ItemComplexity::Simple,
                virtual_scrolling: true,
                filtering: false,
                iterations: 100,
                viewport_size: (80, 24),
            },
            TestConfig {
                name: "Large Dataset - Complex Items".to_string(),
                item_count: 10_000,
                item_complexity: ItemComplexity::Complex,
                virtual_scrolling: true,
                filtering: true,
                iterations: 50,
                viewport_size: (120, 30),
            },
            TestConfig {
                name: "Filtering Performance Test".to_string(),
                item_count: 50_000,
                item_complexity: ItemComplexity::Medium,
                virtual_scrolling: true,
                filtering: true,
                iterations: 25,
                viewport_size: (100, 24),
            },
        ]
    }
}

impl TestDataGenerator {
    /// Generate test items
    pub fn generate_items(count: usize, complexity: ItemComplexity) -> Vec<TestListItem> {
        (0..count)
            .map(|i| Self::generate_item(i, complexity))
            .collect()
    }
    
    /// Generate a single test item
    fn generate_item(index: usize, complexity: ItemComplexity) -> TestListItem {
        match complexity {
            ItemComplexity::Simple => TestListItem {
                id: format!("item-{}", index),
                title: format!("Item {}", index),
                description: format!("Description for item {}", index),
                tags: vec![format!("tag-{}", index % 10)],
                metadata: HashMap::new(),
                complexity,
            },
            ItemComplexity::Medium => TestListItem {
                id: format!("item-{}", index),
                title: format!("Complex Item {} with Longer Title", index),
                description: format!(
                    "This is a more detailed description for item {} with additional information \
                     that might span multiple lines and contain various details about the item.",
                    index
                ),
                tags: vec![
                    format!("category-{}", index % 5),
                    format!("type-{}", index % 3),
                    format!("priority-{}", index % 4),
                ],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("created_at".to_string(), format!("2024-01-{:02}", (index % 30) + 1));
                    meta.insert("author".to_string(), format!("user-{}", index % 100));
                    meta.insert("status".to_string(), format!("status-{}", index % 5));
                    meta
                },
                complexity,
            },
            ItemComplexity::Complex => TestListItem {
                id: format!("complex-item-{}", index),
                title: format!("Very Complex Item {} with Extended Title and Multiple Attributes", index),
                description: format!(
                    "This is an extremely detailed description for complex item {} that contains \
                     a lot of information, multiple paragraphs, and extensive details that would \
                     typically be found in real-world applications with rich data structures. \
                     The description includes technical details, user information, timestamps, \
                     and various other metadata that makes rendering more complex.",
                    index
                ),
                tags: vec![
                    format!("complex-category-{}", index % 8),
                    format!("item-type-{}", index % 6),
                    format!("priority-level-{}", index % 5),
                    format!("region-{}", index % 10),
                    format!("department-{}", index % 7),
                    format!("project-{}", index % 15),
                ],
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("created_at".to_string(), format!("2024-{:02}-{:02}", (index % 12) + 1, (index % 28) + 1));
                    meta.insert("modified_at".to_string(), format!("2024-{:02}-{:02}", (index % 12) + 1, (index % 28) + 1));
                    meta.insert("author".to_string(), format!("complex-user-{}", index % 200));
                    meta.insert("reviewer".to_string(), format!("reviewer-{}", index % 50));
                    meta.insert("status".to_string(), format!("detailed-status-{}", index % 8));
                    meta.insert("version".to_string(), format!("v{}.{}.{}", index % 5, index % 10, index % 100));
                    meta.insert("size".to_string(), format!("{} KB", (index * 13) % 10000));
                    meta.insert("checksum".to_string(), format!("sha256-{:016x}", index));
                    meta.insert("location".to_string(), format!("/path/to/complex/item/{}", index));
                    meta.insert("permissions".to_string(), format!("rwxr--r--"));
                    meta
                },
                complexity,
            },
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_initial_render_ms: 100,    // 100ms for initial render
            max_scroll_time_ms: 16,        // 16ms per scroll (60 FPS)
            max_filter_time_ms: 50,        // 50ms for filtering
            max_memory_mb: 100.0,          // 100MB memory limit
            min_frame_rate: 55.0,          // Minimum 55 FPS
            min_smoothness_score: 80.0,    // 80% smoothness score
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            initial_render_time: Duration::ZERO,
            average_scroll_time: Duration::ZERO,
            filter_time: None,
            memory_usage_mb: 0.0,
            frame_rate: 0.0,
            smoothness_score: 0.0,
            timing_details: HashMap::new(),
        }
    }
}

impl ListItem for TestListItem {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn display_text(&self) -> String {
        format!("{} - {}", self.title, self.description)
    }
    
    fn matches_filter(&self, filter: &str) -> bool {
        let filter_lower = filter.to_lowercase();
        self.title.to_lowercase().contains(&filter_lower)
            || self.description.to_lowercase().contains(&filter_lower)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&filter_lower))
    }
    
    fn sort_key(&self) -> String {
        self.title.clone()
    }
}

/// Test summary
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_time: Duration,
    pub results: Vec<TestResult>,
}

impl TestSummary {
    /// Print detailed test results
    pub fn print_detailed_results(&self) {
        println!("\n📊 Performance Test Results Summary");
        println!("==================================");
        println!("Total tests: {}", self.total_tests);
        println!("Passed: {} ✅", self.passed);
        println!("Failed: {} ❌", self.failed);
        println!("Total time: {:.2}s", self.total_time.as_secs_f64());
        println!("Success rate: {:.1}%", (self.passed as f64 / self.total_tests as f64) * 100.0);
        
        println!("\n📈 Detailed Results:");
        for result in &self.results {
            println!("\n🔍 {}", result.config.name);
            println!("  Items: {}", result.config.item_count);
            println!("  Virtual scrolling: {}", result.config.virtual_scrolling);
            println!("  Status: {}", if result.passed { "✅ PASS" } else { "❌ FAIL" });
            
            if result.passed {
                let m = &result.metrics;
                println!("  Initial render: {:.1}ms", m.initial_render_time.as_millis());
                println!("  Scroll time: {:.1}ms", m.average_scroll_time.as_millis());
                if let Some(filter_time) = m.filter_time {
                    println!("  Filter time: {:.1}ms", filter_time.as_millis());
                }
                println!("  Memory usage: {:.1}MB", m.memory_usage_mb);
                println!("  Frame rate: {:.1}fps", m.frame_rate);
                println!("  Smoothness: {:.1}%", m.smoothness_score);
            } else if let Some(error) = &result.error {
                println!("  Error: {}", error);
            }
        }
    }
}

/// Run performance tests
pub async fn run_performance_tests() -> Result<TestSummary> {
    let mut test_suite = ListPerformanceTest::new();
    test_suite.run_all_tests().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_generation() {
        let simple_items = TestDataGenerator::generate_items(10, ItemComplexity::Simple);
        assert_eq!(simple_items.len(), 10);
        
        let complex_items = TestDataGenerator::generate_items(5, ItemComplexity::Complex);
        assert_eq!(complex_items.len(), 5);
        assert!(complex_items[0].metadata.len() > simple_items[0].metadata.len());
    }
    
    #[test]
    fn test_memory_estimation() {
        let test_suite = ListPerformanceTest::new();
        let items = TestDataGenerator::generate_items(100, ItemComplexity::Simple);
        let memory_mb = test_suite.estimate_memory_usage(&items);
        assert!(memory_mb > 0.0);
    }
    
    #[test]
    fn test_smoothness_calculation() {
        let test_suite = ListPerformanceTest::new();
        
        // Perfect timing (all frames at 60 FPS)
        let perfect_frames = vec![Duration::from_micros(16667); 60];
        let perfect_score = test_suite.calculate_smoothness_score(&perfect_frames);
        assert!(perfect_score > 90.0);
        
        // Variable timing
        let variable_frames = vec![
            Duration::from_micros(16667),
            Duration::from_micros(20000),
            Duration::from_micros(15000),
            Duration::from_micros(18000),
        ];
        let variable_score = test_suite.calculate_smoothness_score(&variable_frames);
        assert!(variable_score < perfect_score);
    }
    
    #[tokio::test]
    async fn test_single_performance_test() {
        let test_suite = ListPerformanceTest::new();
        let config = TestConfig {
            name: "Test".to_string(),
            item_count: 100,
            item_complexity: ItemComplexity::Simple,
            virtual_scrolling: true,
            filtering: false,
            iterations: 5,
            viewport_size: (80, 24),
        };
        
        let result = test_suite.run_single_test(config).await;
        assert!(result.is_ok());
    }
}