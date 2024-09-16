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
			<span class="title">dfrs</span>
		</h1>
		<p class="subtitle">A Programming Language for DiamondFire</p>
		<div>
			<button class="btn-start">
				<svg xmlns="http://www.w3.org/2000/svg" height="32" width="24" viewBox="0 0 384 512"><!--!Font Awesome Free 6.6.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2024 Fonticons, Inc.--><path d="M73 39c-14.8-9.1-33.4-9.4-48.5-.9S0 62.6 0 80L0 432c0 17.4 9.4 33.4 24.5 41.9s33.7 8.1 48.5-.9L361 297c14.3-8.7 23-24.2 23-41s-8.7-32.2-23-41L73 39z"/></svg>
				Getting started
			</button>
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
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.btn-start:hover {
		background-color: lightblue
	}

	.btn-start:active {
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
