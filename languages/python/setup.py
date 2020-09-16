import setuptools

setuptools.setup(
	author_email="root@tangramhq.com",
	author="Tangram",
	classifiers=[
		"Programming Language :: Python",
		"License :: OSI Approved :: MIT License",
		"Operating System :: OS Independent",
	],
	description="Tangram for Python",
	include_package_data=True,
	install_requires=[
		'cffi',
		'requests',
	],
	long_description_content_type="text/markdown",
	long_description=open("README.md", "r").read(),
	name="tangram",
	packages=setuptools.find_packages(),
	url="https://github.com/tangram-hq/tangram/tree/master/languages/python",
	version="0.1.3",
)
