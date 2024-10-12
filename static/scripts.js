async function registerUser() {
  // Get username and password from input fields
  const username = document.getElementById('register_username').value;
  const password = document.getElementById('register_password').value;
  const confirmPassword = document.getElementById('confirm_password').value;

  if (!username || !password || !confirmPassword) {
    alert('Please fill in all fields.');
    return;
  }

  const response = await fetch('/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password, confirm_password: confirmPassword })
  });

  if (response.ok) {
    alert('Registered successfully! You can now log in.');
  } else {
    const errorText = await response.text();
    alert('Registration failed: ' + errorText);
  }
}

async function loginUser() {
  // Get username and password from the input fields
  const username = document.getElementById('login_username').value;
  const password = document.getElementById('login_password').value;

  if (!username || !password) {
    alert('Please enter both username and password to log in.');
    return;
  }

  const response = await fetch('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  });

  if (response.ok) {
    // Redirect to user's page
    window.location.href = `/user_pages/${username}/my_page.html`;
  } else {
    const errorText = await response.text();
    alert('Could not take to user page: ' + errorText);
  }
}

