import { BLOG_POSTS } from "../content/blog-posts";
import { useRouter } from "../router";
import { Helmet } from "react-helmet-async";

export function BlogPage() {
	const { navigate } = useRouter();

	return (
		<>
			<Helmet>
				<title>Blog — DagLock</title>
				<meta name="description" content="DagLock blog — KRC-20 escrow, AI mediation, SilverScript covenants on Kaspa." />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Blog</h1>
					<p>Updates, guides, and deep-dives on DagLock and Kaspa covenant development</p>
				</div>

				<div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
					{BLOG_POSTS.map((post) => (
						<article
							key={post.slug}
							className="offer"
							style={{ cursor: "pointer" }}
							onClick={() => navigate(`/blog/${post.slug}` as any)}
						>
							<div className="offer-top">
								<strong>{post.title}</strong>
								<span className="pill" style={{ fontSize: "11px" }}>
									{post.date}
								</span>
							</div>
							<p style={{ margin: "8px 0", fontSize: "14px", color: "var(--color-text-secondary)" }}>
								{post.excerpt}
							</p>
							<code style={{ fontSize: "12px" }}>Read more →</code>
						</article>
					))}
				</div>
			</div>
		</>
	);
}
