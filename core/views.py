from django.contrib.auth import get_user_model
from django.contrib.auth.decorators import login_required
from django.core.paginator import Paginator
from django.db import models
from django.db.models import Q
from django.http import HttpResponse
from django.shortcuts import render, redirect, get_object_or_404
from django.views.decorators.http import require_http_methods

from .forms import ThoughtForm, UserProfileForm
from .models import Thought, UserProfile, HashTag, Friendship
from .utils import get_top_hashtags, get_friends_from_friendship, get_public_thoughts

user_model = get_user_model()


def index(request):
    if request.method == 'POST':
        form = ThoughtForm(request.POST, author=request.user)
        if form.is_valid():
            form.save()
            return redirect('recent_thoughts')
    else:
        form = ThoughtForm()

    return render(request, 'home.html',
                  {'form': form})


def recent_thoughts(request):
    recent_thoughts_objects = (Thought.objects.exclude(
        author__userprofile__visibility=UserProfile.VisibilityChoices.PRIVATE).exclude(
        author__userprofile__visibility=UserProfile.VisibilityChoices.FRIENDS_ONLY).order_by(
        '-created_at'))[:5]
    return render(request, 'partials/recent_thoughts.html', {'recent_thoughts': recent_thoughts_objects})


def user_thoughts(request, username):
    user = get_object_or_404(user_model, username=username)
    thoughts = Thought.objects.filter(author=user).order_by('-created_at')
    return render(request, 'partials/recent_thoughts.html', {'recent_thoughts': thoughts})


def friends_thoughts(request):
    user = request.user
    if not user.is_authenticated:
        return render(request, 'partials/friends_thoughts.html', {'friends_thoughts': []})

    friends = get_friends_from_friendship(user)
    thoughts = Thought.objects.filter(author__in=friends).order_by('-created_at')

    paginator = Paginator(thoughts, 10)
    page_number = request.GET.get('page')
    thoughts = paginator.get_page(page_number)

    return render(request, 'partials/friends_thoughts.html', {'friends_thoughts': thoughts})


def all_thoughts(request):
    thoughts = get_public_thoughts()
    paginator = Paginator(thoughts, 10)
    page_number = request.GET.get('page')
    thoughts = paginator.get_page(page_number)

    return render(request, 'partials/all_thoughts.html', {'all_thoughts': thoughts})


@login_required
def profile_view(request):
    user = request.user
    profile, created = UserProfile.objects.get_or_create(user=user)
    if request.method == 'POST':
        form = UserProfileForm(request.POST, instance=profile)
        if form.is_valid():
            form.save()
            return redirect('profile')
    else:
        form = UserProfileForm(instance=profile)

    thoughts = Thought.objects.filter(author=user).order_by('-created_at')
    incoming_requests = Friendship.objects.filter(friend=user, accepted=False)
    return render(request, 'profile.html', {'form': form, 'thoughts': thoughts, 'incoming_requests': incoming_requests})


@login_required
@require_http_methods(["DELETE"])
def delete_thought(request, thought_id):
    thought = get_object_or_404(Thought, id=thought_id, author=request.user)
    thought.delete()

    thoughts_count = Thought.objects.filter(author=request.user).count()
    if thoughts_count == 0:
        return HttpResponse("No thoughts yet.", status=200)

    return HttpResponse("", status=200)


def top_hashtags(request):
    hashtags = get_top_hashtags()
    return render(request, 'partials/top_hashtags.html', {'top_hashtags': hashtags})


def hashtag_view(request, tag):
    hashtag = get_object_or_404(HashTag, name=tag)
    thoughts = hashtag.thoughts.all()
    return render(request, 'hashtag_thoughts.html', {'thoughts': thoughts, 'hashtag': hashtag})


def user_profile(request, username):
    user = get_object_or_404(user_model, username=username)
    UserProfile.objects.get_or_create(user=user)
    visibility = user.userprofile.visibility
    if visibility == UserProfile.VisibilityChoices.PRIVATE and user != request.user:
        return HttpResponse("This profile is private", status=403)
    if visibility == UserProfile.VisibilityChoices.FRIENDS_ONLY and user != request.user:
        if not request.user.is_authenticated:
            return redirect('account_login')
        is_friend = Friendship.objects.filter(
            Q(creator=user, friend=request.user) | models.Q(creator=request.user, friend=user)
        ).exists()
        if not is_friend and user != request.user:
            return HttpResponse("This profile is only visible to friends", status=403)

    thoughts = Thought.objects.filter(author=user).order_by('-created_at')
    if request.user.is_authenticated:
        friendship = Friendship.objects.filter(
            Q(creator=request.user, friend=user) |
            Q(creator=user, friend=request.user),
            accepted=True
        ).first()
        pending_request = Friendship.objects.filter(
            Q(creator=request.user, friend=user, accepted=False) |
            Q(creator=user, friend=request.user, accepted=False)
        ).exists()
        can_add_friend = not friendship and not pending_request and user != request.user
    incoming_requests = None
    if request.user == user:
        incoming_requests = Friendship.objects.filter(friend=user, accepted=False)
    return render(request, 'user_profile.html',
                  {'profile_user': user, 'thoughts': thoughts, 'incoming_requests': incoming_requests,
                   'can_add_friend': can_add_friend if request.user.is_authenticated else False})


@login_required()
def send_friend_request(request, username):
    user_to_friend = get_object_or_404(user_model, username=username)
    if request.user != user_to_friend and not Friendship.objects.filter(creator=request.user,
                                                                        friend=user_to_friend).exists():
        Friendship.objects.create(creator=request.user, friend=user_to_friend, accepted=False)
    return redirect('user_profile', username=username)


@login_required()
def accept_friend_request(request, username):
    friendship_request = get_object_or_404(Friendship, creator__username=username, friend=request.user)
    friendship_request.accepted = True
    friendship_request.save()
    return redirect('user_profile', username=request.user.username)


def friend_list(request, username):
    user = get_object_or_404(user_model, username=username)
    friends = get_friends_from_friendship(user)
    return render(request, 'partials/friend_list.html', {'friends': friends})


@login_required()
def incoming_friend_requests(request, username):
    user = get_object_or_404(user_model, username=username)
    if request.user != user:
        return HttpResponse("Unauthorized", status=403)

    incoming_requests = Friendship.objects.filter(friend=user, accepted=False)
    return render(request, 'partials/incoming_friend_requests.html', {
        'incoming_requests': incoming_requests
    })


def thought_detail(request, thought_id):
    thought = get_object_or_404(Thought, id=thought_id)
    return render(request, 'thought_detail.html', {'thought': thought})
