window.onload = () => {
    const form = document.querySelector('#thought-form');
    const audio = document.getElementById('submitSound');

    form.addEventListener('submit', function (event) {
        event.preventDefault();
        if (audio) audio.play();
        spawnConfetti();
    });
}

const spawnConfetti = () => {
    for (let i = 0; i < 100; i++) {
        const confetti = document.createElement('div');
        confetti.className = 'confetti';
        document.body.appendChild(confetti);

        confetti.style.left = `${Math.random() * window.innerWidth}px`;
        confetti.style.top = `0px`;
        confetti.style.backgroundColor = `hsl(${Math.random() * 360}, 100%, 50%)`;
        confetti.style.opacity = `${Math.random()}`;

        // Animation
        const movement = Math.random() * 100 - 50;  // Confetti movement range
        const rotation = Math.random() * 360;      // Rotation in degrees
        const scale = Math.random() * 0.5 + 0.5;   // Scale between 0.5 and 1
        const duration = Math.random() * 2 + 0.1;    // Duration between 0.1 and 2

        confetti.animate([
            {transform: `translateX(0px) translateY(0px) rotate(0deg) scale(1)`, opacity: 1},
            {
                transform: `translateX(${movement}px) translateY(500px) rotate(${rotation}deg) scale(${scale})`,
                opacity: 0
            }
        ], {
            duration: duration * 1000,
            easing: 'linear',
            iterations: 1
        });

        setTimeout(() => {
            confetti.remove();
        }, duration * 1000);
    }
}