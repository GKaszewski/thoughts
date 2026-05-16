import { getAllUsers } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";
import { PaginationNav } from "@/components/pagination-nav";

export default async function AllUsersPage({
  searchParams,
}: {
  searchParams: Promise<{ page?: string }>;
}) {
  const { page: pageStr } = await searchParams;
  const page = parseInt(pageStr ?? "1", 10);
  const usersData = await getAllUsers(page).catch(() => null);

  if (!usersData) {
    return (
      <div className="container mx-auto max-w-2xl p-4 sm:p-6 text-center">
        <h1 className="text-3xl font-bold my-6">All Users</h1>
        <p className="text-muted-foreground">
          Could not load users. Please try again later.
        </p>
      </div>
    );
  }

  const { items, total, per_page } = usersData;
  const totalPages = Math.ceil(total / per_page);

  return (
    <div className="container mx-auto max-w-2xl p-4 sm:p-6">
      <header className="my-6 glass-effect glossy-effect bottom gloss-highlight rounded-md p-4 text-shadow-md">
        <h1 className="text-3xl font-bold">All Users</h1>
        <p className="text-muted-foreground">
          Discover other users on Thoughts.
        </p>
      </header>
      <main>
        <UserListCard users={items} />
        <PaginationNav
          page={page}
          totalPages={totalPages}
          buildHref={(p) => `/users/all?page=${p}`}
        />
      </main>
    </div>
  );
}
