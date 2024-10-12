function checkLogin() {
  // Check if the "user_id" cookie is set
  const cookies = document.cookie.split("; ");
  const userIdCookie = cookies.find(cookie => cookie.startsWith("user_id="));

  if (userIdCookie) {
    alert("DEBUG: You're logged in.");
  } else {
    alert("DEBUG: You are not logged in.");
  }
}

