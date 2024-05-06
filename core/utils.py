from django.db.models import Count, Q

from .models import HashTag, Friendship, Thought, UserProfile


def get_top_hashtags():
    return HashTag.objects.filter(
        thoughts__author__userprofile__visibility=UserProfile.VisibilityChoices.PUBLIC).annotate(
        num_thoughts=Count('thoughts')).order_by('-num_thoughts')[:5]


def get_friends_from_friendship(user):
    if not user.is_authenticated:
        return []
    friendships = Friendship.objects.filter((Q(creator=user) | Q(friend=user)) & Q(accepted=True))
    friends = []
    for friendship in friendships:
        if friendship.creator == user:
            friends.append(friendship.friend)
        else:
            friends.append(friendship.creator)

    return friends


def get_public_thoughts():
    return (Thought.objects.filter(author__userprofile__visibility=UserProfile.VisibilityChoices.PUBLIC).order_by(
        '-created_at'))
