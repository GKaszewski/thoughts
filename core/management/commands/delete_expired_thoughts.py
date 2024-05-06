from django.core.management.base import BaseCommand
from django.utils import timezone
from core.models import Thought


class Command(BaseCommand):
    help = 'Delete expired thoughts'

    def handle(self, *args, **options):
        expired_thoughts = Thought.objects.filter(expires_at__lt=timezone.now())
        count = expired_thoughts.count()
        expired_thoughts.delete()
        self.stdout.write(self.style.SUCCESS(f'Successfully deleted {count} expired thoughts'))
