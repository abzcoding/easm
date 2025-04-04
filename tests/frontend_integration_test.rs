use anyhow::Result;
use fantoccini::{Client, Locator};
use std::time::Duration;

async fn setup_webdriver() -> Result<Client> {
    // Create a new WebDriver client
    let client = Client::new("http://localhost:4444").await?;
    Ok(client)
}

#[tokio::test]
async fn test_frontend_login_and_navigation() -> Result<()> {
    // Start the test by navigating to the login page
    let client = setup_webdriver().await?;

    // Navigate to the login page
    client.goto("http://localhost:8080/login").await?;

    // Wait for the login form to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Fill in login credentials
    client
        .find(Locator::Css("input[type='email']"))
        .await?
        .send_keys("admin@example.com")
        .await?;

    client
        .find(Locator::Css("input[type='password']"))
        .await?
        .send_keys("admin123")
        .await?;

    // Submit the form
    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for redirect to dashboard
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check if we're on the dashboard
    let current_url = client.current_url().await?;
    assert!(
        current_url.path() == "/" || current_url.path() == "/dashboard",
        "Expected to be redirected to dashboard, but URL is {}",
        current_url
    );

    // Navigate to the Assets page
    client
        .find(Locator::XPath("//a[contains(text(), 'Assets')]"))
        .await?
        .click()
        .await?;

    // Wait for assets page to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify we're on the assets page
    let current_url = client.current_url().await?;
    assert!(
        current_url.path().contains("assets"),
        "Expected to be on assets page, but URL is {}",
        current_url
    );

    // Test adding a new asset
    client
        .find(Locator::Css("button[aria-label='Add Asset']"))
        .await?
        .click()
        .await?;

    // Wait for modal to appear
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Fill in asset form
    client
        .find(Locator::Css("input[name='name']"))
        .await?
        .send_keys("Test Integration Asset")
        .await?;

    client
        .find(Locator::Css("input[name='url']"))
        .await?
        .send_keys("https://test-integration.example.com")
        .await?;

    // Select asset type
    client
        .find(Locator::Css("select[name='asset_type']"))
        .await?
        .select_by_value("DOMAIN")
        .await?;

    // Submit the form
    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for asset to be created
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check if the new asset appears in the list
    let asset_exists = client
        .find_all(Locator::XPath(
            "//td[contains(text(), 'Test Integration Asset')]",
        ))
        .await?
        .len()
        > 0;

    assert!(
        asset_exists,
        "Newly created asset doesn't appear in the list"
    );

    // Cleanup by closing the browser
    client.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_frontend_vulnerabilities_page() -> Result<()> {
    // Start the test by logging in
    let client = setup_webdriver().await?;

    // Navigate to the login page
    client.goto("http://localhost:8080/login").await?;

    // Wait for the login form to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Fill in login credentials
    client
        .find(Locator::Css("input[type='email']"))
        .await?
        .send_keys("admin@example.com")
        .await?;

    client
        .find(Locator::Css("input[type='password']"))
        .await?
        .send_keys("admin123")
        .await?;

    // Submit the form
    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for redirect to dashboard
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Navigate to the Vulnerabilities page
    client
        .find(Locator::XPath("//a[contains(text(), 'Vulnerabilities')]"))
        .await?
        .click()
        .await?;

    // Wait for vulnerabilities page to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify we're on the vulnerabilities page
    let current_url = client.current_url().await?;
    assert!(
        current_url.path().contains("vulnerabilities"),
        "Expected to be on vulnerabilities page, but URL is {}",
        current_url
    );

    // Test vulnerability filtering functionality
    // Find filter input
    let filter_input = client
        .find(Locator::Css("input[placeholder*='Search']"))
        .await?;
    filter_input.send_keys("SQL").await?;

    // Wait for filtering to apply
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check if filtering worked
    let visible_vulnerabilities = client
        .find_all(Locator::Css("tr.vulnerability-row"))
        .await?;

    // If there are results, they should be SQL-related
    if !visible_vulnerabilities.is_empty() {
        for vuln in visible_vulnerabilities {
            let vuln_text = vuln.text().await?;
            assert!(
                vuln_text.to_lowercase().contains("sql"),
                "Filtered vulnerability doesn't contain 'SQL': {}",
                vuln_text
            );
        }
    }

    // Cleanup by closing the browser
    client.close().await?;

    Ok(())
}

#[tokio::test]
async fn test_frontend_ui_components() -> Result<()> {
    // Start the test by logging in
    let client = setup_webdriver().await?;

    // Navigate to the login page
    client.goto("http://localhost:8080/login").await?;

    // Wait for the login form to load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check login page UI components
    let app_title = client.find(Locator::Css("h1")).await?.text().await?;
    assert!(
        app_title.contains("EASM") || app_title.contains("External Attack Surface Management"),
        "App title not found"
    );

    // Login
    client
        .find(Locator::Css("input[type='email']"))
        .await?
        .send_keys("admin@example.com")
        .await?;

    client
        .find(Locator::Css("input[type='password']"))
        .await?
        .send_keys("admin123")
        .await?;

    client
        .find(Locator::Css("button[type='submit']"))
        .await?
        .click()
        .await?;

    // Wait for dashboard
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check sidebar navigation
    let sidebar = client.find(Locator::Css("nav")).await?;
    let nav_items = sidebar.find_all(Locator::Css("a")).await?;

    // Verify expected navigation items
    let nav_texts = futures::future::join_all(nav_items.iter().map(|item| item.text())).await;
    let nav_texts: Vec<String> = nav_texts.into_iter().filter_map(Result::ok).collect();

    let expected_items = vec![
        "Dashboard",
        "Assets",
        "Vulnerabilities",
        "Discovery",
        "Technologies",
    ];
    for item in expected_items {
        assert!(
            nav_texts.iter().any(|text| text.contains(item)),
            "Navigation item '{}' not found in sidebar",
            item
        );
    }

    // Test responsive UI by resizing window
    // Set to mobile size
    client.set_window_size(375, 667).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check if hamburger menu appears
    let hamburger_exists = client
        .find_all(Locator::Css("button[aria-label*='menu']"))
        .await?
        .len()
        > 0;
    assert!(hamburger_exists, "Hamburger menu not found in mobile view");

    // Reset window size
    client.set_window_size(1280, 800).await?;

    // Cleanup by closing the browser
    client.close().await?;

    Ok(())
}
