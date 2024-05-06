# Generated by Django 5.0.4 on 2024-05-06 00:42

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('core', '0004_userprofile_birthdate_userprofile_github_and_more'),
    ]

    operations = [
        migrations.CreateModel(
            name='HashTag',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.CharField(max_length=128, unique=True)),
                ('thoughts', models.ManyToManyField(to='core.thought')),
            ],
        ),
    ]
