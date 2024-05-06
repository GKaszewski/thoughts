from django.urls import path

from .views import index, recent_thoughts, profile_view, delete_thought, user_thoughts, top_hashtags, hashtag_view, \
    user_profile, send_friend_request, friend_list, accept_friend_request, incoming_friend_requests, thought_detail, \
    friends_thoughts, all_thoughts, site_stats, search

urlpatterns = [
    path('', index, name='index'),
    path('profile/', profile_view, name='profile'),
    path('profile/<str:username>/', user_profile, name='user_profile'),
    path('recent-thoughts/', recent_thoughts, name='recent_thoughts'),
    path('top-hashtags/', top_hashtags, name='top_hashtags'),
    path('hashtag/<str:tag>/', hashtag_view, name='hashtag_thoughts'),
    path('thoughts/<str:username>/', user_thoughts, name='user_thoughts'),
    path('thoughts/<int:thought_id>/delete', delete_thought, name='delete_thought'),
    path('friend-request/<str:username>', send_friend_request, name='send_friend_request'),
    path('friend-request/accept/<str:username>', accept_friend_request, name='accept_friend_request'),
    path('friends/<str:username>', friend_list, name='friend_list'),
    path('incoming-friend-requests/<str:username>', incoming_friend_requests, name='incoming_friend_requests'),
    path('thought/<int:thought_id>/', thought_detail, name='thought_detail'),
    path('friends-thoughts/', friends_thoughts, name='friends_thoughts'),
    path('all-thoughts/', all_thoughts, name='all_thoughts'),
    path('site-stats/', site_stats, name='site_stats'),
    path('search/', search, name='search'),
]
