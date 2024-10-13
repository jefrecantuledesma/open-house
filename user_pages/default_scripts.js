async function generateInvite() {
  const response = await fetch('/generate_invite', {
    method: 'GET',
    credentials: 'include',
  });

  if (response.ok) {
    const data = await response.json();
    const inviteCode = data.invite_code;
    document.getElementById('inviteLink').innerHTML = `
      <p>Share this code (valid for one use):</p>
      <p>${inviteCode}</p>
    `;
  } else {
    const errorText = await response.text();
    alert('Error generating invite code: ' + errorText);
  }
}

function showFriendsSidebar() {
  const friendsSidebar = document.getElementById('friendsSidebar');
  const overlay = document.getElementById('overlay');
  friendsSidebar.classList.add('show');
  overlay.classList.add('show');

  // Fetch and display friends
  fetchFriends();
}

function closeFriendsSidebar() {
  const friendsSidebar = document.getElementById('friendsSidebar');
  const overlay = document.getElementById('overlay');
  friendsSidebar.classList.remove('show');
  overlay.classList.remove('show');
}


async function fetchFriends() {
  try {
    const response = await fetch('/get_friends', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching friends: ' + errorText);
    } else {
      const friends = await response.json();
      displayFriends(friends);
    }
  } catch (error) {
    alert('Error fetching friends: ' + error.message);
  }
}

function displayFriends(friends) {
  const friendsList = document.getElementById('friendsList');
  friendsList.innerHTML = ''; // Clear existing list

  friends.forEach((friend) => {
    const friendItem = document.createElement('li');
    const friendLink = document.createElement('a');
    friendLink.href = `/user_pages/${friend}/my_page.html`;
    friendLink.textContent = friend;
    friendItem.appendChild(friendLink);
    friendsList.appendChild(friendItem);
  });
}

function toggleFriendForm() {
  const addFriendButton = document.getElementById('addFriendButton');
  const friendLinkInput = document.getElementById('friendLinkInput');
  const submitFriendButton = document.getElementById('submitFriendButton');
  const generateInviteButton = document.querySelector('.header-buttons button:nth-child(1)');
  const showFriendsButton = document.getElementById('showFriendsButton');

  // Toggle visibility and animate the buttons
  if (friendLinkInput.style.display === 'none') {
    friendLinkInput.style.display = 'inline-block';
    submitFriendButton.style.display = 'inline-block';
    showFriendsButton.style.display = 'inline-block';

    // Add classes to animate buttons shifting to the left
    addFriendButton.classList.add('shift-left');
    generateInviteButton.classList.add('shift-left');
    showFriendsButton.classList.add('shift-left');
  } else {
    // Hide the input and submit button and reset button position
    friendLinkInput.style.display = 'none';
    submitFriendButton.style.display = 'none';
    addFriendButton.classList.remove('shift-left');
    generateInviteButton.classList.remove('shift-left');
    showFriendsButton.classList.remove('shift-left');
  }
}

async function addFriend() {
  const inviteCode = document.getElementById('friendLinkInput').value.trim();

  if (!inviteCode) {
    alert('Please enter an invite code.');
    return;
  }

  try {
    const response = await fetch('/add_friend', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ invite_code: inviteCode }),
      credentials: 'include'
    });

    if (response.ok) {
      alert('Friend added successfully!');
      // Hide the friend form after submission
      toggleFriendForm();
      document.getElementById('friendLinkInput').value = '';
    } else {
      const errorText = await response.text();
      alert('Error adding friend: ' + errorText);
    }
  } catch (error) {
    alert('Error adding friend: ' + error.message);
  }
}

function showFilmForm() {
  document.getElementById('filmForm').classList.add('show');
  document.getElementById('showFilmButton').classList.add('active');
}

function hideFilmForm() {
  document.getElementById('filmForm').classList.remove('show');
  document.getElementById('showFilmButton').classList.remove('active');
}

document.getElementById('film-video').addEventListener('change', function() {
  const file = this.files[0];
  const fileNameElement = document.getElementById('film-file-name');
  if (file) {
    fileNameElement.textContent = `Selected: ${file.name}`;
  } else {
    fileNameElement.textContent = 'No file selected';
  }
});

async function uploadFilm() {
  const filmTitle = document.getElementById('film-title').value.trim();
  const fileInput = document.getElementById('film-video');
  const file = fileInput.files[0];

  if (!filmTitle) {
    alert('Please enter a film title.');
    return;
  }

  if (!file) {
    alert('Please select a video file.');
    return;
  }

  // Check file size (max 200MB)
  if (file.size > 200 * 1024 * 1024) {
    alert(`File "${file.name}" is too big (must be under 200MB).`);
    return;
  }

  // Create a FormData object
  const formData = new FormData();
  formData.append('filmTitle', filmTitle);
  formData.append('video', file);

  try {
    const response = await fetch('/upload_film', {
      method: 'POST',
      credentials: 'include',
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error uploading film: ' + errorText);
    } else {
      alert('Film uploaded successfully!');
      // Close the form
      hideFilmForm();
      // Clear the form
      document.getElementById('film-title').value = '';
      fileInput.value = '';
      document.getElementById('film-file-name').textContent = 'No file selected';
      // Refresh the films display
      fetchAllContent();
    }
  } catch (error) {
    alert('Error uploading film: ' + error.message);
  }
}

async function fetchFilms() {
  try {
    const response = await fetch('/get_films', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching films: ' + errorText);
    } else {
      const films = await response.json();
      displayFilms(films);
    }
  } catch (error) {
    alert('Error fetching films: ' + error.message);
  }
}

function displayFilms(films) {
  const filmsDiv = document.getElementById('films');
  filmsDiv.innerHTML = '';

  films.forEach((film) => {
    const filmSection = document.createElement('section');
    const filmTitle = document.createElement('h3');
    filmTitle.textContent = film.title;
    filmSection.appendChild(filmTitle);

    const videoElement = document.createElement('video');
    videoElement.src = film.video_path;
    videoElement.controls = true;
    filmSection.appendChild(videoElement);

    filmsDiv.appendChild(filmSection);
  });
}

function showAudioForm() {
  document.getElementById('audioForm').classList.add('show');
  document.getElementById('showAudioButton').classList.add('active');
}

function hideAudioForm() {
  document.getElementById('audioForm').classList.remove('show');
  document.getElementById('showAudioButton').classList.remove('active');
}

document.getElementById('audio-file').addEventListener('change', function() {
  const file = this.files[0];
  const fileNameElement = document.getElementById('audio-file-name');
  if (file) {
    fileNameElement.textContent = `Selected: ${file.name}`;
  } else {
    fileNameElement.textContent = 'No file selected';
  }
});

async function uploadAudio() {
  const audioTitle = document.getElementById('audio-title').value.trim();
  const fileInput = document.getElementById('audio-file');
  const file = fileInput.files[0];

  if (!audioTitle) {
    alert('Please enter an audio title.');
    return;
  }

  if (!file) {
    alert('Please select an audio file.');
    return;
  }

  // Check file size (max 50MB)
  if (file.size > 50 * 1024 * 1024) {
    alert(`File "${file.name}" is too big (must be under 50MB).`);
    return;
  }

  // Create a FormData object
  const formData = new FormData();
  formData.append('audioTitle', audioTitle);
  formData.append('audio', file);

  try {
    const response = await fetch('/upload_audio', {
      method: 'POST',
      credentials: 'include',
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error uploading audio: ' + errorText);
    } else {
      alert('Audio uploaded successfully!');
      // Close the form
      hideAudioForm();
      // Clear the form
      document.getElementById('audio-title').value = '';
      fileInput.value = '';
      document.getElementById('audio-file-name').textContent = 'No file selected';
      // Refresh the audios display
      fetchAllContent();
    }
  } catch (error) {
    alert('Error uploading audio: ' + error.message);
  }
}

async function fetchAudios() {
  try {
    const response = await fetch('/get_audios', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching audios: ' + errorText);
    } else {
      const audios = await response.json();
      displayAudios(audios);
    }
  } catch (error) {
    alert('Error fetching audios: ' + error.message);
  }
}

function displayAudios(audios) {
  const audiosDiv = document.getElementById('audios');
  audiosDiv.innerHTML = '';

  audios.forEach((audio) => {
    const audioSection = document.createElement('section');
    const audioTitle = document.createElement('h3');
    audioTitle.textContent = audio.title;
    audioSection.appendChild(audioTitle);

    const audioElement = document.createElement('audio');
    audioElement.src = audio.audio_path;
    audioElement.controls = true;
    audioSection.appendChild(audioElement);

    audiosDiv.appendChild(audioSection);
  });
}

function editPage() {
  const sidebar = document.getElementById('editSidebar');
  const overlay = document.getElementById('overlay');
  sidebar.classList.add('show');
  overlay.classList.add('show');

  // Populate the input fields with current titles
  const exhibitTitleInput = document.getElementById('edit-exhibit-title');
  const mainTitleInput = document.getElementById('edit-main-title');

  // Get current titles from the page
  const exhibitTitle = document.querySelector('header h1');
  const mainTitle = document.querySelector('main h2');

  exhibitTitleInput.value = exhibitTitle.textContent;
  mainTitleInput.value = mainTitle.textContent;
}

function closeSidebar() {
  const sidebar = document.getElementById('editSidebar');
  const overlay = document.getElementById('overlay');
  sidebar.classList.remove('show');
  overlay.classList.remove('show');
}

function saveChanges() {
  const exhibitTitleInput = document.getElementById('edit-exhibit-title');
  const mainTitleInput = document.getElementById('edit-main-title');

  const newExhibitTitle = exhibitTitleInput.value.trim();
  const newMainTitle = mainTitleInput.value.trim();

  if (!newExhibitTitle || !newMainTitle) {
    alert('Please fill in both fields.');
    return;
  }

  // Update the page titles
  const exhibitTitle = document.querySelector('header h1');
  const mainTitle = document.querySelector('main h2');

  exhibitTitle.textContent = newExhibitTitle;
  mainTitle.textContent = newMainTitle;

  // Close the sidebar
  closeSidebar();

  // Send changes to server to save
  saveChangesToServer(newExhibitTitle, newMainTitle);
}

async function saveChangesToServer(exhibitTitle, mainTitle) {
  try {
    const response = await fetch('/save_changes', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ exhibitTitle, mainTitle })
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error saving changes: ' + errorText);
    } else {
      alert('Changes saved successfully!');
    }
  } catch (error) {
    alert('Error saving changes: ' + error.message);
  }
}

document.addEventListener("DOMContentLoaded", function() {
  const header = document.querySelector("header");
  const mainContent = document.querySelector("main");
  const headerHeight = header.offsetHeight;
  mainContent.style.paddingTop = headerHeight + "px";
  fetchAllContent();

  document.getElementById('gallery-images').addEventListener('change', function() {
    const files = this.files;
    const fileCount = document.getElementById('file-count');
    if (files.length > 0) {
      fileCount.textContent = `${files.length} file(s) selected`;
    } else {
      fileCount.textContent = 'No files selected';
    }
  });
});

function showGalleryForm() {
  document.getElementById('galleryForm').classList.add('show');
  document.getElementById('showGalleryButton').classList.add('active');
}

function hideGalleryForm() {
  document.getElementById('galleryForm').classList.remove('show');
  document.getElementById('showGalleryButton').classList.remove('active');
}

async function uploadGallery() {
  const galleryTitle = document.getElementById('gallery-title').value.trim();
  const files = document.getElementById('gallery-images').files;

  if (!galleryTitle) {
    alert('Please enter a gallery title.');
    return;
  }

  if (files.length === 0) {
    alert('Please select at least one image.');
    return;
  }

  if (files.length > 20) {
    alert('You can upload a maximum of 20 images.');
    return;
  }

  // Create a FormData object
  const formData = new FormData();
  formData.append('galleryTitle', galleryTitle);

  for (let i = 0; i < files.length; i++) {
    const file = files[i];

    // Check file size (max 10MB)
    if (file.size > 10 * 1024 * 1024) {
      alert(`File "${file.name}" is too big (must be under 10MB).`);
      return;
    }

    // Append the file to the form data
    formData.append('images', file);
  }

  try {
    const response = await fetch('/upload_gallery', {
      method: 'POST',
      credentials: 'include',
      body: formData,
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error uploading gallery: ' + errorText);
    } else {
      alert('Gallery uploaded successfully!');
      // Close the form
      hideGalleryForm();
      // Clear the form
      document.getElementById('gallery-title').value = '';
      document.getElementById('gallery-images').value = '';
      document.getElementById('file-count').textContent = 'No files selected';
      // Refresh the galleries display
      fetchAllContent();
    }
  } catch (error) {
    alert('Error uploading gallery: ' + error.message);
  }
}

async function fetchGalleries() {
  try {
    const response = await fetch('/get_galleries', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching galleries: ' + errorText);
    } else {
      const galleries = await response.json();
      displayGalleries(galleries);
    }
  } catch (error) {
    alert('Error fetching galleries: ' + error.message);
  }
}

function displayGalleries(galleries) {
  const galleriesDiv = document.getElementById('galleries');
  galleriesDiv.innerHTML = '';

  galleries.forEach((gallery) => {
    const gallerySection = document.createElement('section');
    const galleryTitle = document.createElement('h3');
    galleryTitle.textContent = gallery.title;
    gallerySection.appendChild(galleryTitle);

    const galleryDiv = document.createElement('div');
    galleryDiv.classList.add('gallery');

    gallery.images.forEach((imagePath) => {
      const imgElement = document.createElement('img');
      imgElement.src = imagePath;
      galleryDiv.appendChild(imgElement);
    });

    gallerySection.appendChild(galleryDiv);
    galleriesDiv.appendChild(gallerySection);
  });
}

function showTextPostForm() {
  document.getElementById('textPostForm').classList.add('show');
  document.getElementById('showTextPostButton').classList.add('active');
}

function hideTextPostForm() {
  document.getElementById('textPostForm').classList.remove('show');
  document.getElementById('showTextPostButton').classList.remove('active');
}

async function uploadTextPost() {
  const postTitle = document.getElementById('text-post-title').value.trim();
  const postContent = document.getElementById('text-post-content').value.trim();

  if (!postTitle) {
    alert('Please enter a post title.');
    return;
  }

  if (!postContent) {
    alert('Please enter some content for your post.');
    return;
  }

  const postData = {
    title: postTitle,
    content: postContent
  };

  try {
    const response = await fetch('/upload_text_post', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(postData)
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error uploading text post: ' + errorText);
    } else {
      alert('Text post uploaded successfully!');
      // Close the form
      hideTextPostForm();
      // Clear the form
      document.getElementById('text-post-title').value = '';
      document.getElementById('text-post-content').value = '';
      // Refresh the text posts display
      fetchAllContent();
    }
  } catch (error) {
    alert('Error uploading text post: ' + error.message);
  }
}

async function fetchTextPosts() {
  try {
    const response = await fetch('/get_text_posts', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching text posts: ' + errorText);
    } else {
      const textPosts = await response.json();
      displayTextPosts(textPosts);
    }
  } catch (error) {
    alert('Error fetching text posts: ' + error.message);
  }
}

function displayTextPosts(textPosts) {
  const textPostsDiv = document.getElementById('text-posts');
  textPostsDiv.innerHTML = '';

  textPosts.forEach((post) => {
    const postSection = document.createElement('section');
    const postTitle = document.createElement('h3');
    postTitle.textContent = post.title;
    postSection.appendChild(postTitle);

    const postContent = document.createElement('p');
    postContent.textContent = post.content;
    postSection.appendChild(postContent);

    textPostsDiv.appendChild(postSection);
  });
}

async function fetchAllContent() {
  try {
    const response = await fetch('/get_all_content', {
      method: 'GET',
      credentials: 'include',
    });

    if (!response.ok) {
      const errorText = await response.text();
      alert('Error fetching content: ' + errorText);
    } else {
      const contentItems = await response.json();
      displayAllContent(contentItems);
    }
  } catch (error) {
    alert('Error fetching content: ' + error.message);
  }
}

function displayAllContent(contentItems) {
  const contentFeed = document.getElementById('content-feed');
  contentFeed.innerHTML = '';

  contentItems.forEach((item) => {
    const contentSection = document.createElement('section');
    const contentTitle = document.createElement('h3');

    switch (item.type) {
      case 'TextPost':
        contentTitle.textContent = item.title;
        contentSection.appendChild(contentTitle);

        const postContent = document.createElement('p');
        postContent.textContent = item.content;
        contentSection.appendChild(postContent);
        break;

      case 'Gallery':
        contentTitle.textContent = item.title;
        contentSection.appendChild(contentTitle);

        const galleryDiv = document.createElement('div');
        galleryDiv.classList.add('gallery');

        item.images.forEach((imagePath) => {
          const imgElement = document.createElement('img');
          imgElement.src = imagePath;
          galleryDiv.appendChild(imgElement);
        });

        contentSection.appendChild(galleryDiv);
        break;

      case 'Film':
        contentTitle.textContent = item.title;
        contentSection.appendChild(contentTitle);

        const videoElement = document.createElement('video');
        videoElement.src = item.video_path;
        videoElement.controls = true;
        contentSection.appendChild(videoElement);
        break;

      case 'Audio':
        contentTitle.textContent = item.title;
        contentSection.appendChild(contentTitle);

        const audioElement = document.createElement('audio');
        audioElement.src = item.audio_path;
        audioElement.controls = true;
        contentSection.appendChild(audioElement);
        break;

      default:
        console.error('Unknown content type:', item.type);
    }

    contentFeed.appendChild(contentSection);
  });
}
