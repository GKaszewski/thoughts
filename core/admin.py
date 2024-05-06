from django.contrib import admin
from .models import Thought, UserProfile, Friendship

admin.site.register(Thought)
admin.site.register(UserProfile)
admin.site.register(Friendship)
