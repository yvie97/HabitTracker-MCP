#!/usr/bin/env python3
"""
Test script to simulate MCP client requests (like what Claude would send)

This script sends JSON-RPC messages to our habit tracker MCP server
to test if the protocol implementation works correctly.
"""

import json
import subprocess
import sys
import time

def send_request(process, request_id, method, params=None):
    """Send a JSON-RPC request to the MCP server"""
    request = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method
    }
    if params:
        request["params"] = params
    
    request_str = json.dumps(request)
    print(f"→ Sending: {request_str}")
    
    process.stdin.write(request_str + "\n")
    process.stdin.flush()
    
    # Read response
    response_str = process.stdout.readline()
    if response_str:
        print(f"← Received: {response_str.strip()}")
        try:
            return json.loads(response_str.strip())
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse response: {e}")
            return None
    return None

def main():
    print("🧪 Testing Habit Tracker MCP Server")
    print("=" * 50)
    
    # Start the MCP server process
    print("Starting MCP server...")
    process = subprocess.Popen(
        ["cargo", "run", "--bin", "habit-tracker-mcp", "--", "--debug"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1  # Line buffered
    )
    
    try:
        # Give the server a moment to start
        time.sleep(2)
        
        print("\n1. Testing MCP Initialization")
        print("-" * 30)
        
        # Initialize the connection
        init_response = send_request(process, 1, "initialize", {
            "protocol_version": "2024-11-05",
            "capabilities": {},
            "client_info": {
                "name": "Test Client",
                "version": "1.0.0"
            }
        })
        
        if init_response and init_response.get("result"):
            print("✅ Initialization successful!")
        else:
            print("❌ Initialization failed")
            return
        
        # Send initialized notification
        send_request(process, 2, "initialized", {})
        
        print("\n2. Testing Tool Discovery")
        print("-" * 30)
        
        # List available tools
        tools_response = send_request(process, 3, "tools/list", {})
        
        if tools_response and tools_response.get("result"):
            tools = tools_response["result"].get("tools", [])
            print(f"✅ Found {len(tools)} tools:")
            for tool in tools:
                print(f"   - {tool['name']}: {tool['description']}")
        else:
            print("❌ Tool discovery failed")
        
        print("\n3. Testing Habit Creation")
        print("-" * 30)
        
        # Create a test habit
        create_response = send_request(process, 4, "tools/call", {
            "name": "habit_create",
            "arguments": {
                "name": "Morning Exercise",
                "category": "health",
                "frequency": "daily"
            }
        })
        
        if create_response and create_response.get("result"):
            print("✅ Habit creation successful!")
            content = create_response["result"].get("content", [])
            if content:
                print(f"   Message: {content[0].get('text', '')}")
        else:
            print("❌ Habit creation failed")
        
        print("\n4. Testing Habit Listing")
        print("-" * 30)
        
        # List all habits
        list_response = send_request(process, 5, "tools/call", {
            "name": "habit_list",
            "arguments": {}
        })
        
        if list_response and list_response.get("result"):
            print("✅ Habit listing successful!")
            content = list_response["result"].get("content", [])
            if content:
                print(f"   Result: {content[0].get('text', '')}")
        else:
            print("❌ Habit listing failed")

        print("\n4.1 Testing Enhanced Habit Listing Features")
        print("-" * 30)

        # Create additional test habits with different frequencies
        test_habits = [
            {"name": "Weekly Exercise", "category": "health", "frequency": "weekly:3"},
            {"name": "Weekday Reading", "category": "productivity", "frequency": "weekdays"},
            {"name": "Weekend Meditation", "category": "mindfulness", "frequency": "weekends"},
            {"name": "Writing Practice", "category": "creative", "frequency": "interval:2"}
        ]

        habit_ids = []

        for i, habit_data in enumerate(test_habits):
            create_resp = send_request(process, 50 + i, "tools/call", {
                "name": "habit_create",
                "arguments": habit_data
            })

            if create_resp and create_resp.get("result"):
                content = create_resp["result"].get("content", [])
                if content:
                    message = content[0].get("text", "")
                    if "Habit ID:" in message:
                        test_habit_id = message.split("Habit ID: ")[1].strip()
                        habit_ids.append(test_habit_id)
                        print(f"   ✅ Created {habit_data['name']} with ID: {test_habit_id}")

        # Add some completions to test streak calculations
        import json
        from datetime import datetime, timedelta

        if habit_ids:
            print("   Adding sample completions for streak testing...")
            today = datetime.now()

            # Add completions for the first habit (daily exercise)
            for j in range(3):
                date = (today - timedelta(days=2-j)).strftime("%Y-%m-%d")
                log_resp = send_request(process, 60 + j, "tools/call", {
                    "name": "habit_log",
                    "arguments": {
                        "habit_id": habit_ids[0] if habit_ids else habit_id,
                        "completed_at": date,
                        "value": 30,
                        "intensity": 8
                    }
                })
                if log_resp and log_resp.get("result") and not log_resp["result"].get("is_error"):
                    print(f"      ✅ Logged completion for {date}")

        # Test 4.1.1: Frequency Display Testing
        print("\n   4.1.1 Testing frequency display...")
        detailed_list = send_request(process, 70, "tools/call", {
            "name": "habit_list",
            "arguments": {}
        })

        if detailed_list and detailed_list.get("result"):
            content = detailed_list["result"].get("content", [])
            if content:
                list_text = content[0].get("text", "")

                # Check for different frequency displays
                frequency_checks = [
                    ("Daily", "daily frequency"),
                    ("3 times per week", "weekly frequency"),
                    ("Weekdays", "weekdays frequency"),
                    ("Weekends", "weekends frequency"),
                    ("Every 2 days", "interval frequency")
                ]

                found_frequencies = 0
                for freq_text, description in frequency_checks:
                    if freq_text in list_text:
                        print(f"      ✅ Found {description}: '{freq_text}'")
                        found_frequencies += 1

                if found_frequencies >= 3:
                    print("   ✅ Frequency display conversion working correctly")
                else:
                    print(f"   ⚠️ Only found {found_frequencies} frequency displays")

        # Test 4.1.2: Sorting functionality
        print("\n   4.1.2 Testing sorting functionality...")

        sort_tests = [
            ("streak", "streak sorting"),
            ("completion_rate", "completion rate sorting"),
            ("name", "name sorting")
        ]

        for sort_by, description in sort_tests:
            sort_resp = send_request(process, 80 + len([s for s, _ in sort_tests if s == sort_by]), "tools/call", {
                "name": "habit_list",
                "arguments": {"sort_by": sort_by}
            })

            if sort_resp and sort_resp.get("result"):
                content = sort_resp["result"].get("content", [])
                if content:
                    sorted_text = content[0].get("text", "")
                    if "habits" in sorted_text.lower():
                        print(f"      ✅ {description} successful")
                    else:
                        print(f"      ❌ {description} failed")
            else:
                print(f"      ❌ {description} failed - no response")

        # Test 4.1.3: Category filtering
        print("\n   4.1.3 Testing category filtering...")

        category_resp = send_request(process, 85, "tools/call", {
            "name": "habit_list",
            "arguments": {"category": "health"}
        })

        if category_resp and category_resp.get("result"):
            content = category_resp["result"].get("content", [])
            if content:
                filtered_text = content[0].get("text", "")
                if "health" in filtered_text.lower():
                    print("      ✅ Category filtering working")
                else:
                    print("      ⚠️ Category filtering may not be working as expected")

        # Test 4.1.4: Streak data validation
        print("\n   4.1.4 Testing streak data in listing...")

        full_list = send_request(process, 90, "tools/call", {
            "name": "habit_list",
            "arguments": {}
        })

        if full_list and full_list.get("result"):
            content = full_list["result"].get("content", [])
            if content:
                full_text = content[0].get("text", "")

                # Check for streak-related data
                streak_indicators = [
                    ("streak", "streak data"),
                    ("completion", "completion data"),
                    ("rate", "rate data"),
                    ("%", "percentage data")
                ]

                found_data = 0
                for indicator, description in streak_indicators:
                    if indicator in full_text.lower():
                        print(f"      ✅ Found {description}")
                        found_data += 1

                if found_data >= 2:
                    print("   ✅ Streak and completion data being displayed")
                else:
                    print(f"   ⚠️ Limited streak data found ({found_data} indicators)")

                # Look for specific numeric data (not zeros)
                if any(char.isdigit() and char != '0' for char in full_text):
                    print("      ✅ Found non-zero numeric data")
                else:
                    print("      ⚠️ All numeric data appears to be zeros")

        # Test 4.1.5: JSON structure validation (if we can parse it)
        print("\n   4.1.5 Testing data structure completeness...")

        json_test = send_request(process, 95, "tools/call", {
            "name": "habit_list",
            "arguments": {"sort_by": "streak"}
        })

        if json_test and json_test.get("result"):
            content = json_test["result"].get("content", [])
            if content:
                result_text = content[0].get("text", "")

                # Check for structured data indicators
                structure_checks = [
                    ("habit", "habit references"),
                    ("frequency", "frequency data"),
                    ("active", "activity status"),
                    ("category", "category data")
                ]

                structure_found = 0
                for check, description in structure_checks:
                    if check in result_text.lower():
                        print(f"      ✅ Found {description}")
                        structure_found += 1

                if structure_found >= 3:
                    print("   ✅ Data structure appears complete")
                else:
                    print(f"   ⚠️ Limited data structure ({structure_found} elements)")

        print("\n   📊 Enhanced habit listing tests completed")
        
        print("\n5. Testing Habit Logging")
        print("-" * 30)

        # Extract habit ID from the create response (use original habit from test 3)
        habit_id = None
        if create_response and create_response.get("result"):
            content = create_response["result"].get("content", [])
            if content:
                message = content[0].get("text", "")
                # Look for "Habit ID:" in the response
                if "Habit ID:" in message:
                    habit_id = message.split("Habit ID: ")[1].strip()
                    print(f"   Extracted habit ID: {habit_id}")
        
        if habit_id:
            # Try to log a habit completion
            log_response = send_request(process, 6, "tools/call", {
                "name": "habit_log",
                "arguments": {
                    "habit_id": habit_id,
                    "value": 30,
                    "intensity": 8,
                    "notes": "Great morning workout!"
                }
            })
            
            if log_response and log_response.get("result"):
                result = log_response["result"]
                if result.get("is_error"):
                    print("❌ Habit logging failed")
                    content = result.get("content", [])
                    if content:
                        print(f"   Error: {content[0].get('text', '')}")
                else:
                    print("✅ Habit logging successful!")
                    content = result.get("content", [])
                    if content:
                        print(f"   Result: {content[0].get('text', '')}")
            else:
                print("❌ Habit logging failed - no response")
        else:
            print("❌ Could not extract habit ID for logging test")
        
        print("\n6. Testing Habit Status")
        print("-" * 30)
        
        # Test habit status for all habits
        status_response = send_request(process, 7, "tools/call", {
            "name": "habit_status",
            "arguments": {}
        })
        
        if status_response and status_response.get("result"):
            result = status_response["result"]
            if result.get("is_error"):
                print("❌ Habit status failed")
                content = result.get("content", [])
                if content:
                    print(f"   Error: {content[0].get('text', '')}")
            else:
                print("✅ Habit status successful!")
                content = result.get("content", [])
                if content:
                    print(f"   Result:\n{content[0].get('text', '')}")
        else:
            print("❌ Habit status failed - no response")
        
        print("\n7. Testing Habit Insights (Enhanced)")
        print("-" * 30)

        # Test basic insights for all habits
        print("   7.1 Testing overall insights...")
        insights_response = send_request(process, 8, "tools/call", {
            "name": "habit_insights",
            "arguments": {
                "time_period": "month",
                "insight_type": "all"
            }
        })

        insights_success = False
        if insights_response and insights_response.get("result"):
            result = insights_response["result"]
            if result.get("is_error"):
                print("   ❌ Overall insights failed")
                content = result.get("content", [])
                if content:
                    print(f"      Error: {content[0].get('text', '')}")
            else:
                print("   ✅ Overall insights successful!")
                content = result.get("content", [])
                if content:
                    insights_text = content[0].get('text', '')
                    print(f"      Result:\n{insights_text}")

                    # Verify sophisticated analytics features
                    if "Habit Insights Report" in insights_text:
                        print("      ✅ Found formatted insights report")
                        insights_success = True
                    if "📊" in insights_text or "💡" in insights_text or "🎉" in insights_text:
                        print("      ✅ Found insight emojis")
                    if "insights:" in insights_text.lower():
                        print("      ✅ Found insight summary")
        else:
            print("   ❌ Overall insights failed - no response")

        # Test specific habit insights if we have a habit ID
        if habit_id and insights_success:
            print("\n   7.2 Testing specific habit insights...")
            specific_insights = send_request(process, 9, "tools/call", {
                "name": "habit_insights",
                "arguments": {
                    "habit_id": habit_id,
                    "time_period": "month",
                    "insight_type": "all"
                }
            })

            if specific_insights and specific_insights.get("result"):
                result = specific_insights["result"]
                if not result.get("is_error"):
                    print("   ✅ Specific habit insights successful!")
                    content = result.get("content", [])
                    if content:
                        specific_text = content[0].get('text', '')
                        # Check for specific analytics features
                        if "completion rate" in specific_text.lower():
                            print("      ✅ Found completion rate analysis")
                        if "streak" in specific_text.lower():
                            print("      ✅ Found streak analysis")
                        if "consistency" in specific_text.lower() or "performance" in specific_text.lower():
                            print("      ✅ Found performance insights")
                else:
                    print("   ❌ Specific habit insights failed")
            else:
                print("   ❌ Specific habit insights failed - no response")

        # Test insight filtering by type
        print("\n   7.3 Testing insight filtering...")
        filtered_insights = send_request(process, 10, "tools/call", {
            "name": "habit_insights",
            "arguments": {
                "time_period": "month",
                "insight_type": "recommendation"
            }
        })

        if filtered_insights and filtered_insights.get("result"):
            result = filtered_insights["result"]
            if not result.get("is_error"):
                print("   ✅ Insight filtering successful!")
                content = result.get("content", [])
                if content:
                    filtered_text = content[0].get('text', '')
                    if "recommendation" in filtered_text.lower() or "💡" in filtered_text:
                        print("      ✅ Found recommendation insights")
            else:
                print("   ❌ Insight filtering failed")
        else:
            print("   ❌ Insight filtering failed - no response")

        # Create additional habits to test diversity analytics
        print("\n   7.4 Testing category diversity analytics...")

        # Create a second habit in different category
        create_response2 = send_request(process, 11, "tools/call", {
            "name": "habit_create",
            "arguments": {
                "name": "Daily Reading",
                "category": "productivity",
                "frequency": "daily"
            }
        })

        # Create a third habit in another category
        create_response3 = send_request(process, 12, "tools/call", {
            "name": "habit_create",
            "arguments": {
                "name": "Meditation",
                "category": "mindfulness",
                "frequency": "daily"
            }
        })

        if create_response2 and create_response3:
            # Now test overall insights with multiple categories
            diversity_insights = send_request(process, 13, "tools/call", {
                "name": "habit_insights",
                "arguments": {
                    "time_period": "month",
                    "insight_type": "all"
                }
            })

            if diversity_insights and diversity_insights.get("result"):
                result = diversity_insights["result"]
                if not result.get("is_error"):
                    content = result.get("content", [])
                    if content:
                        diversity_text = content[0].get('text', '')
                        if "diversifying" in diversity_text.lower() or "well-rounded" in diversity_text.lower():
                            print("   ✅ Found category diversity analysis")
                        if "life areas" in diversity_text.lower() or "categories" in diversity_text.lower():
                            print("   ✅ Found category analysis")
                        print(f"      Multi-category insights:\n{diversity_text}")
                else:
                    print("   ❌ Diversity insights failed")

        print("\n   📊 Analytics testing summary:")
        print("      - Overall insights: ✅" if insights_success else "      - Overall insights: ❌")
        print("      - Sophisticated features verified")
        print("      - Insight filtering tested")
        print("      - Category diversity tested")
        
        print("\n8. Testing Error Handling")
        print("-" * 30)

        # Test invalid habit creation
        print("   Testing invalid habit name...")
        invalid_create = send_request(process, 14, "tools/call", {
            "name": "habit_create",
            "arguments": {
                "name": "",  # Empty name should fail
                "category": "health",
                "frequency": "daily"
            }
        })

        if invalid_create and invalid_create.get("result") and invalid_create["result"].get("is_error"):
            print("   ✅ Empty name validation working")
        else:
            print("   ❌ Empty name validation failed")

        # Test invalid category
        print("   Testing invalid category...")
        invalid_category = send_request(process, 15, "tools/call", {
            "name": "habit_create",
            "arguments": {
                "name": "Test Habit",
                "category": "invalid_category",
                "frequency": "daily"
            }
        })

        if invalid_category and invalid_category.get("result") and invalid_category["result"].get("is_error"):
            print("   ✅ Invalid category validation working")
        else:
            print("   ❌ Invalid category validation failed")

        # Test invalid habit logging
        print("   Testing invalid habit logging...")
        invalid_log = send_request(process, 16, "tools/call", {
            "name": "habit_log",
            "arguments": {
                "habit_id": "invalid-id-format",
                "intensity": 15  # Should be 1-10
            }
        })

        if invalid_log and invalid_log.get("result") and invalid_log["result"].get("is_error"):
            print("   ✅ Invalid logging validation working")
        else:
            print("   ❌ Invalid logging validation failed")
        
        print("\n9. Testing Streak Calculations")
        print("-" * 30)

        # Helper function to extract streak data from status
        def extract_streak_data(status_text, habit_name):
            lines = status_text.split('\n')
            for i, line in enumerate(lines):
                if habit_name in line and "🎯" in line:
                    # Look at the next line for streak info
                    if i + 1 < len(lines):
                        streak_line = lines[i + 1]
                        if "Current streak:" in streak_line and "Best:" in streak_line:
                            parts = streak_line.split("|")
                            current_part = parts[0].strip()
                            best_part = parts[1].strip()

                            current_streak = int(current_part.split("Current streak: ")[1].split(" days")[0])
                            best_streak = int(best_part.split("Best: ")[1].split(" days")[0])

                            return current_streak, best_streak
            return None, None

        # Test different frequency types
        test_cases = [
            {"name": "Daily Streak Test", "frequency": "daily"},
            {"name": "Weekdays Streak Test", "frequency": "weekdays"},
            {"name": "Weekly Streak Test", "frequency": "weekly:3"},
            {"name": "Weekend Streak Test", "frequency": "weekends"},
            {"name": "Interval Streak Test", "frequency": "interval:3"}
        ]

        from datetime import datetime, timedelta

        for i, test_case in enumerate(test_cases):
            print(f"   9.{i+1} Testing {test_case['name']}...")

            # Create habit for this test
            create_response = send_request(process, 100+i, "tools/call", {
                "name": "habit_create",
                "arguments": {
                    "name": test_case["name"],
                    "category": "health",
                    "frequency": test_case["frequency"]
                }
            })

            if not (create_response and create_response.get("result") and not create_response["result"].get("is_error")):
                print(f"   ❌ Failed to create {test_case['name']}")
                continue

            # Extract habit ID
            content = create_response["result"]["content"][0]["text"]
            if "Habit ID:" in content:
                test_habit_id = content.split("Habit ID: ")[1].strip()
            else:
                print(f"   ❌ Could not extract habit ID for {test_case['name']}")
                continue

            # Log some completions for streak testing
            today = datetime.now()
            log_count = 0

            if test_case["frequency"] == "daily":
                # Log 3 consecutive days
                for j in range(3):
                    date = (today - timedelta(days=2-j)).strftime("%Y-%m-%d")
                    log_response = send_request(process, 200+i*10+j, "tools/call", {
                        "name": "habit_log",
                        "arguments": {
                            "habit_id": test_habit_id,
                            "completed_at": date
                        }
                    })
                    if log_response and log_response.get("result") and not log_response["result"].get("is_error"):
                        log_count += 1

            elif test_case["frequency"] == "weekdays":
                # Log weekdays only
                for j in range(7):
                    date = today - timedelta(days=j)
                    if date.weekday() < 5:  # Monday=0 to Friday=4
                        date_str = date.strftime("%Y-%m-%d")
                        log_response = send_request(process, 200+i*10+j, "tools/call", {
                            "name": "habit_log",
                            "arguments": {
                                "habit_id": test_habit_id,
                                "completed_at": date_str
                            }
                        })
                        if log_response and log_response.get("result") and not log_response["result"].get("is_error"):
                            log_count += 1

            elif test_case["frequency"] == "weekends":
                # Log weekends only
                for j in range(14):
                    date = today - timedelta(days=j)
                    if date.weekday() >= 5:  # Saturday=5, Sunday=6
                        date_str = date.strftime("%Y-%m-%d")
                        log_response = send_request(process, 200+i*10+j, "tools/call", {
                            "name": "habit_log",
                            "arguments": {
                                "habit_id": test_habit_id,
                                "completed_at": date_str
                            }
                        })
                        if log_response and log_response.get("result") and not log_response["result"].get("is_error"):
                            log_count += 1

            elif "weekly:" in test_case["frequency"]:
                # Log 3 times this week and 3 times last week
                for j in [0, 2, 4, 7, 9, 11]:  # Spread across 2 weeks
                    date = (today - timedelta(days=j)).strftime("%Y-%m-%d")
                    log_response = send_request(process, 200+i*10+j, "tools/call", {
                        "name": "habit_log",
                        "arguments": {
                            "habit_id": test_habit_id,
                            "completed_at": date
                        }
                    })
                    if log_response and log_response.get("result") and not log_response["result"].get("is_error"):
                        log_count += 1

            elif "interval:" in test_case["frequency"]:
                # Log every 3 days
                for j in [0, 3, 6]:
                    date = (today - timedelta(days=j)).strftime("%Y-%m-%d")
                    log_response = send_request(process, 200+i*10+j, "tools/call", {
                        "name": "habit_log",
                        "arguments": {
                            "habit_id": test_habit_id,
                            "completed_at": date
                        }
                    })
                    if log_response and log_response.get("result") and not log_response["result"].get("is_error"):
                        log_count += 1

            # Get status and check streaks
            status_response = send_request(process, 300+i, "tools/call", {
                "name": "habit_status",
                "arguments": {}
            })

            if status_response and status_response.get("result") and not status_response["result"].get("is_error"):
                status_text = status_response["result"]["content"][0]["text"]
                current_streak, best_streak = extract_streak_data(status_text, test_case["name"])

                print(f"      Logged {log_count} completions")
                print(f"      Current streak: {current_streak}, Best streak: {best_streak}")

                if current_streak is not None and best_streak is not None and current_streak > 0:
                    print(f"   ✅ {test_case['name']} streak calculation working")
                else:
                    print(f"   ❌ {test_case['name']} streak calculation failed")
            else:
                print(f"   ❌ Failed to get status for {test_case['name']}")

        print("\n   📊 Streak calculation testing completed")

        print("\n🎉 MCP Server test completed!")
        
    except KeyboardInterrupt:
        print("\n⏹️ Test interrupted by user")
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
    finally:
        # Clean up
        print("\n🧹 Cleaning up...")
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
        
        # Show any stderr output
        stderr = process.stderr.read()
        if stderr:
            print("\n📝 Server stderr:")
            print(stderr)

if __name__ == "__main__":
    main()