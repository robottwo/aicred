#!/bin/bash

# Test script for the complete tagging and labeling system
# This script runs comprehensive integration tests and validates the system

set -e

echo "🚀 Starting AICred Tagging and Labeling System Integration Tests"
echo "=================================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run tests and report results
run_test_suite() {
    local test_name=$1
    local test_command=$2
    
    print_status "Running $test_name..."
    
    if eval $test_command; then
        print_success "$test_name passed"
        return 0
    else
        print_error "$test_name failed"
        return 1
    fi
}

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to increment counters
record_test_result() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ $1 -eq 0 ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Clean up any existing test data
cleanup_test_data() {
    print_status "Cleaning up existing test data..."
    rm -rf ~/.config/aicred/tags.yaml
    rm -rf ~/.config/aicred/labels.yaml
    rm -rf ~/.config/aicred/tag_assignments.yaml
    rm -rf ~/.config/aicred/label_assignments.yaml
}

# Test 1: Core Data Structure Tests
print_status "=== Phase 1: Core Data Structure Tests ==="
run_test_suite "Core Models Unit Tests" "cd core && cargo test models::tag --lib"
record_test_result $?

run_test_suite "Core Models Unit Tests" "cd core && cargo test models::label --lib"
record_test_result $?

run_test_suite "Core Models Unit Tests" "cd core && cargo test models::tag_assignment --lib"
record_test_result $?

run_test_suite "Core Models Unit Tests" "cd core && cargo test models::label_assignment --lib"
record_test_result $?

# Test 2: Integration Tests
print_status "=== Phase 2: Integration Tests ==="
run_test_suite "Tagging Integration Tests" "cd core && cargo test tagging_integration_tests"
record_test_result $?

run_test_suite "CLI Integration Tests" "cd cli && cargo test tagging_integration_tests"
record_test_result $?

# Test 3: CLI Command Tests
print_status "=== Phase 3: CLI Command Tests ==="
run_test_suite "CLI Tag Commands" "cd cli && cargo test tags"
record_test_result $?

run_test_suite "CLI Label Commands" "cd cli && cargo test labels"
record_test_result $?

# Test 4: Build and Compilation Tests
print_status "=== Phase 4: Build and Compilation Tests ==="
run_test_suite "Core Library Build" "cd core && cargo build"
record_test_result $?

run_test_suite "CLI Build" "cd cli && cargo build"
record_test_result $?

run_test_suite "GUI Build" "cd gui && npm run build"
record_test_result $?

# Test 5: End-to-End Workflow Tests
print_status "=== Phase 5: End-to-End Workflow Tests ==="

# Create test environment
TEST_HOME=$(mktemp -d)
export HOME=$TEST_HOME

# Test tag lifecycle
run_test_suite "Tag Lifecycle Test" "
    cd cli && \
    cargo run -- tags add test-tag --color '#ff0000' --description 'Test tag' && \
    cargo run -- tags list | grep -q 'test-tag' && \
    cargo run -- tags update test-tag --color '#00ff00' && \
    cargo run -- tags remove test-tag --force
"
record_test_result $?

# Test label lifecycle
run_test_suite "Label Lifecycle Test" "
    cd cli && \
    cargo run -- labels add test-label --color '#00ff00' --description 'Test label' && \
    cargo run -- labels list | grep -q 'test-label' && \
    cargo run -- labels update test-label --color '#ff8800' && \
    cargo run -- labels remove test-label --force
"
record_test_result $?

# Test assignment workflow
run_test_suite "Assignment Workflow Test" "
    cd cli && \
    cargo run -- tags add assign-test-tag && \
    cargo run -- labels add assign-test-label && \
    cargo run -- tags assign assign-test-tag --instance test-instance && \
    cargo run -- labels assign assign-test-label --instance test-instance && \
    cargo run -- tags unassign assign-test-tag --instance test-instance && \
    cargo run -- labels unassign assign-test-label --instance test-instance && \
    cargo run -- tags remove assign-test-tag --force && \
    cargo run -- labels remove assign-test-label --force
"
record_test_result $?

# Test 6: Data Persistence Tests
print_status "=== Phase 6: Data Persistence Tests ==="

run_test_suite "Data Persistence Test" "
    cd cli && \
    # Create data
    cargo run -- tags add persistence-tag && \
    cargo run -- labels add persistence-label && \
    # Verify files exist
    [ -f '$TEST_HOME/.config/aicred/tags.yaml' ] && \
    [ -f '$TEST_HOME/.config/aicred/labels.yaml' ] && \
    # Clear memory and reload
    cargo run -- tags list | grep -q 'persistence-tag' && \
    cargo run -- labels list | grep -q 'persistence-label' && \
    # Cleanup
    cargo run -- tags remove persistence-tag --force && \
    cargo run -- labels remove persistence-label --force
"
record_test_result $?

# Test 7: Validation Tests
print_status "=== Phase 7: Validation Tests ==="

run_test_suite "Validation Error Test" "
    cd cli && \
    # Test invalid tag creation
    ! cargo run -- tags add '' 2>/dev/null && \
    # Test duplicate tag
    cargo run -- tags add duplicate-test && \
    ! cargo run -- tags add duplicate-test 2>/dev/null && \
    # Cleanup
    cargo run -- tags remove duplicate-test --force
"
record_test_result $?

run_test_suite "Label Uniqueness Test" "
    cd cli && \
    # Create label and assign it
    cargo run -- labels add unique-test-label && \
    cargo run -- labels assign unique-test-label --instance instance-1 && \
    # Try to assign same label to different instance (should fail)
    ! cargo run -- labels assign unique-test-label --instance instance-2 2>/dev/null && \
    # Cleanup
    cargo run -- labels unassign unique-test-label --instance instance-1 && \
    cargo run -- labels remove unique-test-label --force
"
record_test_result $?

# Test 8: Performance Tests
print_status "=== Phase 8: Performance Tests ==="

run_test_suite "Large Dataset Test" "
    cd cli && \
    # Create 100 tags quickly
    for i in {1..100}; do
        cargo run -- tags add perf-tag-\$i >/dev/null 2>&1 || true
    done && \
    # Test listing performance
    time cargo run -- tags list >/dev/null 2>&1 && \
    # Cleanup
    for i in {1..100}; do
        cargo run -- tags remove perf-tag-\$i --force >/dev/null 2>&1 || true
    done
"
record_test_result $?

# Test 9: Error Handling Tests
print_status "=== Phase 9: Error Handling Tests ==="

run_test_suite "Error Handling Test" "
    cd cli && \
    # Test non-existent operations
    ! cargo run -- tags remove non-existent-tag 2>/dev/null && \
    ! cargo run -- labels remove non-existent-label 2>/dev/null && \
    ! cargo run -- tags assign non-existent-tag --instance test 2>/dev/null && \
    ! cargo run -- labels assign non-existent-label --instance test 2>/dev/null
"
record_test_result $?

# Test 10: GUI Integration Tests (if GUI builds successfully)
print_status "=== Phase 10: GUI Integration Tests ==="

if command -v npm >/dev/null 2>&1; then
    run_test_suite "GUI Build Test" "cd gui && npm run build"
    record_test_result $?
    
    run_test_suite "GUI Type Check" "cd gui && npm run type-check"
    record_test_result $?
else
    print_warning "npm not found, skipping GUI tests"
fi

# Cleanup
cleanup_test_data
rm -rf $TEST_HOME

# Final Report
echo ""
echo "=================================================================="
echo "🏁 Test Results Summary"
echo "=================================================================="
echo -e "Total Tests: ${BLUE}$TOTAL_TESTS${NC}"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo ""
    print_success "🎉 All tests passed! The tagging and labeling system is working correctly."
    exit 0
else
    echo ""
    print_error "❌ Some tests failed. Please review the output above."
    exit 1
fi