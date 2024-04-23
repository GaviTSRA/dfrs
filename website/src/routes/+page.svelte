<script>
	import logo from '$lib/images/logo.png';
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	<div class="top">
		<img src={logo}>
		<h1>
			<span class="title">df.rs</span>
		</h1>
		<p class="subtitle">A DiamondFire Scripting Language</p>
		<div>
			<button class="btn-start">Getting started</button>
			<button class="btn-doc">Documentation</button>
		</div>
	</div>
	<div>
		<div>

		</div>
		<div>
			<p>Easily create events, functions and processes from text</p>
		</div>
	</div>
</section>

<style>
	.btn-start {
		background-color: rgb(0, 241, 0);
	}

	.btn-start:hover {
		background-color: rgb(73, 255, 73)
	}

	.btn-start:active {
		background-color: white;
	}

	.btn-doc {
		background-color: rgb(92, 92, 255);
	}

	.btn-doc:hover {
		background-color: rgb(150, 150, 255);
	}

	.btn-doc:active {
		background-color: white;
	}

	button {
		border-style:none;
		font-size: 1.5rem;
		padding: 1.5rem 3rem;
		border-radius: 10px;
		margin: 0 2rem;
	}

	img {
		width: 10rem;
	}

	section {
		width: 100%;
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 0.6;
		background: radial-gradient(rgb(0, 0, 0) 20%, transparent 100%);
		background-position: 0% 0%;
	}

	.top {
		background: radial-gradient(rgb(49, 49, 49) 8%, transparent 8%);
		background-position: 0% 0%;
		background-size: 5vmin 5vmin;
		width: 100%;
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}

	h1 {
		width: 100%;
		margin:0;
	}

	.title {
		font-size: 10rem;
		font-weight: bold;
		background-color: #f3ec78;
    	background-image: linear-gradient(90deg, #f3ec78, #af4261);
		-webkit-background-clip: text;
		-moz-background-clip: text;
		-webkit-text-fill-color: transparent; 
		-moz-text-fill-color: transparent;
	}

	.subtitle {
		color: aqua;
		font-size: 3rem;
		margin-top:0;
		padding-top:0;
	}
</style>
