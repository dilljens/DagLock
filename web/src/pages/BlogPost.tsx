import { BLOG_POSTS } from "../content/blog-posts";
import { useRouter } from "../router";
import { Helmet } from "react-helmet-async";

export function BlogPost({ slug }: { slug: string }) {
	const { navigate } = useRouter();

	const post = BLOG_POSTS.find((p) => p.slug === slug);

	if (!post) {
		return (
			<div style={{ textAlign: "center", padding: "48px 0" }}>
				<h2>Post not found</h2>
				<p className="muted">The blog post you're looking for doesn't exist.</p>
				<button className="button primary" onClick={() => navigate("/blog" as any)}>
					Back to blog
				</button>
			</div>
		);
	}

	return (
		<>
			<Helmet>
				<title>{post.title} — DagLock Blog</title>
				<meta name="description" content={post.excerpt} />
			</Helmet>
			<div>
				<button className="button" onClick={() => navigate("/blog" as any)} style={{ marginBottom: "16px" }}>
					← Back to blog
				</button>

				<article style={{ maxWidth: "700px" }}>
					<header style={{ marginBottom: "24px" }}>
						<h1 style={{ margin: "0 0 8px" }}>{post.title}</h1>
						<p className="muted" style={{ margin: 0, fontSize: "14px" }}>{post.date}</p>
					</header>

					<div
						className="blog-content"
						dangerouslySetInnerHTML={{ __html: post.content }}
					/>
				</article>
			</div>
		</>
	);
}
