use crate::monday::{MondayClient, MondayUser};
use anyhow::Result;
use std::io;

pub async fn handle_delete_command(
    client: &MondayClient,
    user: &MondayUser,
    delete_id: String,
    yes: bool,
    verbose: bool,
) -> Result<()> {
    println!("\n=== Delete Claim Item ===");
    println!("User: {} ({})", user.name, user.email);
    println!("Item ID to delete: {}", delete_id);

    // First, try to get the item details to show the user what they're deleting
    if verbose {
        println!("🔍 Fetching item details...");
    }

    match client.get_item_by_id(&delete_id, verbose).await {
        Ok(Some(item)) => {
            println!("\n📋 Item Details:");
            println!("  Name: {}", item.name.as_deref().unwrap_or("Unnamed"));
            println!("  ID: {}", item.id.as_deref().unwrap_or("Unknown"));

            // Show relevant column values
            if !item.column_values.is_empty() {
                println!("  Columns:");
                for col in &item.column_values {
                    if let Some(col_id) = &col.id {
                        let column_title = match col_id.as_str() {
                            "date4" => "Date",
                            "status" => "Status",
                            "text__1" => "Customer",
                            "text8__1" => "Work Item",
                            "numbers__1" => "Hours",
                            _ => continue, // Skip less important columns
                        };

                        if let Some(value) = &col.value {
                            if !value.is_empty() && value != "null" {
                                // Try to parse JSON values for better display
                                if let Ok(parsed_value) =
                                    serde_json::from_str::<serde_json::Value>(value)
                                {
                                    if let Some(date_obj) = parsed_value.get("date") {
                                        if let Some(date_str) = date_obj.as_str() {
                                            println!("    {}: {}", column_title, date_str);
                                            continue;
                                        }
                                    }
                                    if let Some(status_index) = parsed_value.get("index") {
                                        if let Some(index_num) = status_index.as_u64() {
                                            let status_name =
                                                crate::map_activity_value_to_name(index_num as u8);
                                            println!("    {}: {}", column_title, status_name);
                                            continue;
                                        }
                                    }
                                }
                                println!("    {}: {}", column_title, value);
                            }
                        } else if let Some(text) = &col.text {
                            if !text.is_empty() && text != "null" {
                                println!("    {}: {}", column_title, text);
                            }
                        }
                    }
                }
            }
        }
        Ok(None) => {
            println!("❌ Item with ID '{}' not found.", delete_id);
            return Ok(());
        }
        Err(e) => {
            println!("⚠️  Could not fetch item details: {}", e);
            println!("Proceeding with deletion based on ID only...");
        }
    }

    // Ask for confirmation unless -y flag is used
    if !yes {
        println!("\n🗑️  Are you sure you want to delete this item?");
        println!("This action cannot be undone! (y/N)");

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;

        if confirmation.trim().to_lowercase() != "y" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    // Perform the deletion
    println!("\n🔄 Deleting item...");
    match client.delete_item(&delete_id, verbose).await {
        Ok(_) => {
            println!("✅ Item deleted successfully!");
        }
        Err(e) => {
            println!("❌ Failed to delete item: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
