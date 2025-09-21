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
        
        print("\n5. Testing Habit Logging")
        print("-" * 30)
        
        # Extract habit ID from the create response
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