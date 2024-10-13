function toggleForm(formType) {
  const registerForm = document.getElementById('registerForm');
  const loginForm = document.getElementById('loginForm');
  const registerButton = document.querySelector('.button-left');
  const loginButton = document.querySelector('.button-right');

  if (formType === 'register') {
    // Toggle the register form and add "active" class to the Register button
    registerForm.classList.toggle('show');
    registerButton.classList.toggle('active');

    // Ensure the login form is hidden and remove "active" class from the Login button
    loginForm.classList.remove('show');
    loginButton.classList.remove('active');
  } else if (formType === 'login') {
    // Toggle the login form and add "active" class to the Login button
    loginForm.classList.toggle('show');
    loginButton.classList.toggle('active');

    // Ensure the register form is hidden and remove "active" class from the Register button
    registerForm.classList.remove('show');
    registerButton.classList.remove('active');
  }
}


async function registerUser() {
  // Get username and password from input fields
  const username = document.getElementById('register_username').value;
  const password = document.getElementById('register_password').value;
  const confirmPassword = document.getElementById('confirm_password').value;
  const registerSuccess = document.getElementById('registerSuccess');
  const registerForm = document.getElementById('registerForm');
  const registerButton = document.querySelector('.button-left');

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
    // Display the "Success" message next to the "Register" button
    registerSuccess.style.display = 'inline';

    // Automatically hide the registration form after 2 seconds
    setTimeout(() => {
      registerForm.classList.remove('show');
      registerButton.classList.remove('active');
      registerSuccess.style.display = 'none';  // Hide success message after form collapses
    }, 2000);
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
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  });

  if (response.ok) {
    // Read the redirect URL from the response body
    const redirectUrl = await response.text();
    window.location.href = redirectUrl;
  } else {
    const errorText = await response.text();
    alert('Could not take to user page: ' + errorText);
  }
}

// Canvas Background Animation
const canvas = document.getElementById('backgroundCanvas');
const ctx = canvas.getContext('2d');
let width, height;
let animationFrameId;

function resizeCanvas() {
  width = canvas.width = window.innerWidth;
  height = canvas.height = window.innerHeight;
}

window.addEventListener('resize', resizeCanvas);
resizeCanvas();

// Neon lines animation with fog effect
let lines = [];
const lineCount = 10;

class Line {
  constructor() {
    this.reset(true);
  }

  reset(offsceen = false) {
    // Start offscreen
    if (offsceen) {
      const side = Math.random() < 0.5 ? 'left' : 'top';
      if (side === 'left') {
        this.x = -Math.random() * width;
        this.y = Math.random() * height;
      } else {
        this.x = Math.random() * width;
        this.y = -Math.random() * height;
      }
    } else {
      this.x = Math.random() * width;
      this.y = Math.random() * height;
    }
    this.length = Math.random() * 200 + 100;
    this.speed = Math.random() * 4 + 2;
    this.angle = Math.atan2(height / 2 - this.y, width / 2 - this.x) + (Math.random() * Math.PI) / 8;
    this.alpha = Math.random() * 0.5 + 0.5;
  }

  update() {
    this.x += this.speed * Math.cos(this.angle);
    this.y += this.speed * Math.sin(this.angle);

    if (
      this.x > width + this.length ||
      this.y > height + this.length ||
      this.x < -this.length ||
      this.y < -this.length
    ) {
      this.reset(true);
    }
  }

  draw() {
    ctx.strokeStyle = `rgba(0, 255, 255, ${this.alpha})`;
    ctx.lineWidth = 4;
    ctx.shadowColor = 'rgba(0, 255, 255, 0.9)';
    ctx.shadowBlur = 40;
    ctx.beginPath();
    ctx.moveTo(this.x, this.y);
    ctx.lineTo(
      this.x - this.length * Math.cos(this.angle),
      this.y - this.length * Math.sin(this.angle)
    );
    ctx.stroke();
    ctx.shadowBlur = 0; // Reset shadowBlur
  }
}

// Fog effect using particles
let fogParticles = [];
const fogParticleCount = 100;

class FogParticle {
  constructor() {
    this.reset();
  }

  reset() {
    this.x = Math.random() * width;
    this.y = height + Math.random() * 100; // Start slightly below the canvas
    this.radius = Math.random() * 100 + 10; // Smaller particles
    this.alpha = Math.random() * 0.1 + 0.1;
    this.speedY = -(Math.random() * 0.4 + 0.5); // Move upwards
    this.speedX = Math.random() * 0.4 - 0.2; // Slight horizontal movement
    this.life = Math.random() * 400 + 100;
    this.age = 0;
  }

  update() {
    this.x += this.speedX;
    this.y += this.speedY;
    this.age++

    if (this.x > width + this.radius) this.x = -this.radius;
    if (this.x < -this.radius) this.x = width + this.radius;
    if (this.y > height + this.radius) this.y = -this.radius;
    if (this.y < -this.radius) this.y = height + this.radius;

    if (this.age > this.life) {
      this.alpha -= 0.005;
    }

    // Reset particle if it becomes invisible
    if (this.alpha <= 0) {
      this.reset();
    }
  }

  draw() {
    const gradient = ctx.createRadialGradient(
      this.x, this.y, 0,
      this.x, this.y, this.radius
    );
    gradient.addColorStop(0, `rgba(255, 0, 255, ${this.alpha})`);
    gradient.addColorStop(1, 'rgba(255, 0, 255, 0)');

    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(this.x, this.y, this.radius, 0, Math.PI * 2);
    ctx.fill();
  }
}

// Initialize lines and fog particles
for (let i = 0; i < lineCount; i++) {
  lines.push(new Line());
}

for (let i = 0; i < fogParticleCount; i++) {
  fogParticles.push(new FogParticle());
}

function animate() {
  ctx.clearRect(0, 0, width, height);

  // Draw fog particles (background fog)
  fogParticles.forEach((particle) => {
    particle.update();
    particle.draw();
  });

  // Draw lines (lasers)
  lines.forEach((line) => {
    line.update();
    line.draw();
  });

  // Draw fog particles over lines (foreground fog)
  fogParticles.forEach((particle) => {
    particle.update();
    particle.draw();
  });

  animationFrameId = requestAnimationFrame(animate);
}

animate();

