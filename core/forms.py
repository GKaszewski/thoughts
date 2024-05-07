from django import forms
from .models import Thought, UserProfile


class ThoughtForm(forms.ModelForm):
    lifespan_days = forms.IntegerField(min_value=1, max_value=14,
                                       label="Lifespan in days", required=False,
                                       widget=forms.NumberInput(attrs={
                                           'class': "w-16 bg-slate-200 appearance-none outline-none p-2 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none",
                                           'max': "14"}))

    text = forms.CharField(
        widget=forms.Textarea(
            attrs={'rows': 4, 'placeholder': "What's on your mind?",
                   'maxlength': '128',
                   'class': "bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none"}),
        label="")

    class Meta:
        model = Thought
        fields = ['text', 'lifespan_days']

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
    custom_css = forms.CharField(
        required=False,
        widget=forms.Textarea(
            attrs={'rows': 5,
                   'class': "bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none"}),
    )
    about_html = forms.CharField(
        required=False,
        widget=forms.Textarea(
            attrs={'rows': 5,
                   'class': "bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none"}),
    )

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
            'location': forms.TextInput(attrs={
                'class': 'bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none'}),
            'birthdate': forms.DateInput(attrs={'type': 'date',
                                                'class': 'bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none'}),
            'website': forms.URLInput(attrs={'placeholder': 'https://example.com',
                                             'class': 'bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none'}),
            'github': forms.URLInput(attrs={'placeholder': 'https://github.com/username',
                                            'class': 'bg-slate-200 resize-none appearance-none outline-none placeholder:text-black py-3 px-4 block w-full border-gray-200 text-sm text-black focus:border-indigo-500 focus:ring-indigo-500 disabled:opacity-50 disabled:pointer-events-none'}),
            'visibility': forms.Select(),
        }
