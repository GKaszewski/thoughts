let displayEditProfile = false;

window.onload = () => {
    const editProfileDiv = document.getElementById('edit-profile');
    const originalDisplay = editProfileDiv.style.display;
    const editProfileButton = document.getElementById('edit-profile-button');
    editProfileDiv.style.display = 'none';

    editProfileButton.addEventListener('click', () => {
        if (displayEditProfile) {
            editProfileDiv.style.display = 'none';
            displayEditProfile = false;
        } else {
            editProfileDiv.style.display = originalDisplay;
            displayEditProfile = true;
        }
    });
}