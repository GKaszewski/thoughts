import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";

interface UserAvatarProps {
  src?: string | null;
  alt?: string | null;
  className?: string;
}

export function UserAvatar({ src, alt, className }: UserAvatarProps) {
  const initial = alt?.trim()[0]?.toUpperCase() ?? "?";

  return (
    <Avatar className={cn("avatar-gradient", className)}>
      {src && (
        <AvatarImage
          className="object-cover object-center"
          src={src}
          alt={alt ?? "User avatar"}
        />
      )}
      <AvatarFallback className="avatar-gradient text-white font-bold text-sm">
        {initial}
      </AvatarFallback>
    </Avatar>
  );
}
