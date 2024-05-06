from django import forms
from .models import Thought, UserProfile


class ThoughtForm(forms.ModelForm):
    lifespan_days = forms.IntegerField(min_value=1, initial=7, help_text="Number of days the thought will be alive",
                                       label="Lifespan in days")

    class Meta:
        model = Thought
        fields = ['text', 'lifespan_days']
        labels = {'text': 'Thought'}
        widgets = {'text': forms.Textarea(attrs={'rows': 3})}

    def __init__(self, *args, **kwargs):
        self.author = kwargs.pop('author', None)
        super().__init__(*args, **kwargs)

    def save(self, commit=True):
        thought = super().save(commit=False)
        thought.author = self.author
        if not thought.author:
            raise ValueError('Author is required')
        if commit:
            thought.set_expiration(self.cleaned_data.get('lifespan_days', None))
            thought.save()
        return thought


class UserProfileForm(forms.ModelForm):
    class Meta:
        model = UserProfile
        fields = ['custom_css', 'about_html', 'website', 'location', 'birthdate', 'github', 'visibility']
        labels = {
            'custom_css': 'Custom CSS',
            'about_html': 'About HTML',
            'website': 'Website',
            'location': 'Location',
            'birthdate': 'Birthdate',
            'github': 'GitHub',
            'visibility': 'Visibility',
        }
        widgets = {
            'custom_css': forms.Textarea(attrs={'rows': 5}),
            'about_html': forms.Textarea(attrs={'rows': 5}),
            'birthdate': forms.DateInput(attrs={'type': 'date'}),
            'website': forms.URLInput(attrs={'placeholder': 'https://example.com'}),
            'github': forms.URLInput(attrs={'placeholder': 'https://github.com/username'}),
            'visibility': forms.Select(attrs={'class': 'form-select'}),
        }
