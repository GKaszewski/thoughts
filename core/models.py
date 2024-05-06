from django.contrib.auth import get_user_model
from django.db import models

user_model = get_user_model()


class Thought(models.Model):
    text = models.CharField(max_length=128)
    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)
    author = models.ForeignKey(user_model, on_delete=models.CASCADE)
    expires_at = models.DateTimeField(null=True, blank=True)

    def __str__(self):
        return f'{self.text} - {self.author}'

    def extract_hashtags(self):
        return set(part[1:] for part in self.text.split() if part.startswith('#'))

    def set_expiration(self, days):
        if not days:
            return
        from django.utils import timezone
        self.expires_at = timezone.now() + timezone.timedelta(days=days)

    def save(
            self, force_insert=False, force_update=False, using=None, update_fields=None
    ):
        super().save(force_insert, force_update, using, update_fields)
        hashtags = self.extract_hashtags()
        for tag in hashtags:
            hashtag, created = HashTag.objects.get_or_create(name=tag.lower())
            hashtag.thoughts.add(self)


class UserProfile(models.Model):
    class VisibilityChoices(models.TextChoices):
        PUBLIC = 'public', 'Public'
        PRIVATE = 'private', 'Private'
        FRIENDS_ONLY = 'friends_only', 'Friends Only'

    user = models.OneToOneField(user_model, on_delete=models.CASCADE)
    visibility = models.CharField(
        max_length=32, choices=VisibilityChoices.choices, default=VisibilityChoices.PUBLIC
    )
    custom_css = models.TextField(blank=True, null=True)
    about_html = models.TextField(blank=True, null=True)
    website = models.URLField(blank=True, null=True)
    location = models.CharField(max_length=64, blank=True, null=True)
    birthdate = models.DateField(blank=True, null=True)
    github = models.URLField(blank=True, null=True)

    def clean_about_html(self):
        import nh3
        allowed_tags = nh3.ALLOWED_TAGS
        allowed_tags.update(['marquee', 'blink'])
        if not self.about_html:
            return
        self.about_html = nh3.clean(self.about_html, tags=allowed_tags)

    def save(
            self, force_insert=False, force_update=False, using=None, update_fields=None
    ):
        self.clean_about_html()
        super().save(force_insert, force_update, using, update_fields)

    def __str__(self):
        return f'{self.user} Profile'


class HashTag(models.Model):
    name = models.CharField(max_length=128, unique=True)
    thoughts = models.ManyToManyField(Thought)

    def __str__(self):
        return f'#{self.name}'


class Friendship(models.Model):
    creator = models.ForeignKey(user_model, related_name="friendship_creator_set", on_delete=models.CASCADE)
    friend = models.ForeignKey(user_model, related_name="friend_set", on_delete=models.CASCADE)
    created_at = models.DateTimeField(auto_now_add=True)
    accepted = models.BooleanField(default=False)

    class Meta:
        constraints = [
            models.UniqueConstraint(fields=['creator', 'friend'], name='unique_friendship')
        ]

    def __str__(self):
        status = "friends with" if self.accepted else "pending friendship request with"
        return f"{self.creator.username} is {status} {self.friend.username}"
