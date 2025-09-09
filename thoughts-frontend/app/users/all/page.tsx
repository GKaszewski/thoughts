import { getAllUsers } from "@/lib/api";
import { UserListCard } from "@/components/user-list-card";
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination";

export default async function AllUsersPage({
  searchParams,
}: {
  searchParams: { page?: string };
}) {
  const page = parseInt(searchParams.page ?? "1", 10);
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

  const { items, totalPages } = usersData;

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
        {totalPages > 1 && (
          <Pagination className="mt-8">
            <PaginationContent>
              <PaginationItem>
                <PaginationPrevious
                  href={page > 1 ? `/users/all?page=${page - 1}` : "#"}
                  aria-disabled={page <= 1}
                />
              </PaginationItem>
              <PaginationItem>
                <PaginationNext
                  href={page < totalPages ? `/users/all?page=${page + 1}` : "#"}
                  aria-disabled={page >= totalPages}
                />
              </PaginationItem>
            </PaginationContent>
          </Pagination>
        )}
      </main>
    </div>
  );
}
